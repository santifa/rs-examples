extern crate libc;
extern crate chrono;
extern crate flate2;

use std::fs::File;
use std::io::Read;
use std::fs::OpenOptions;
use std::os::unix::io::IntoRawFd;

use libc::{uint32_t, c_int, c_ulong};
use chrono::Timelike;
use flate2::read::GzDecoder;

/// Some flags we need to get the appropriate
/// informations from the kernel.
static FBIOGET_VSCREENINFO: u64 = 0x4600;

/// define foreground and background color
static BG: u32 = 0x1D2021;
static FG: u32 = 0xA99A84;

/// Consolefonts psf2 format is stored in little endian ordering.
/// The data is transformed from by 4 bytes from the buffer data.
pub fn from_le(data: &[u8], index: usize) -> u32 {
    ((data[index] as u32) <<  0) +
    ((data[index+1] as u32) <<  8) +
    ((data[index+2] as u32) << 16) +
    ((data[index+3] as u32) << 24)
}

/// This is taken for compliance reasons.
/// Also comments are kept from the c code
#[repr(C)]
pub struct fb_bitfield {
	pub offset: uint32_t, /* beginning of bitfield	*/
	pub length: uint32_t, /* length of bitfield		*/
	pub msb_right: uint32_t, /* != 0 : Most significant bit is right */
}

impl Default for fb_bitfield{
    fn default() -> fb_bitfield {
        fb_bitfield {
            offset: 0,
            length: 0,
            msb_right: 0,
        }
    }
}

/// This struct represents the rust counter-part
/// to the one defined in <kernel/fb.h>.
/// Most of the fields are unused by here for
/// compliance and completeness.
/// The comments are kept from the original c code.
#[repr(C)]
pub struct fb_var_screeninfo {
    pub xres: uint32_t, /* visible resolution	*/
	  pub yres: uint32_t,
	  pub xres_virtual: uint32_t,	/* virtual resolution	*/
	  pub yres_virtual: uint32_t,
	  pub xoffset: uint32_t, /* offset from virtual to visible */
	  pub yoffset: uint32_t, /* resolution */

	  pub bits_per_pixel: uint32_t,	/* guess what */
	  pub grayscale: uint32_t, /* 0 = color, 1 = grayscale,  >1 = FOURCC */
	  pub red: fb_bitfield, /* bitfield in fb mem if true color, */
	  pub green: fb_bitfield,	/* else only length is significant */
	  pub blue: fb_bitfield,
	  pub transp: fb_bitfield, /* transparency */

	  pub nonstd: uint32_t, /* != 0 Non standard pixel format */
	  pub activate: uint32_t, /* see FB_ACTIVATE_* */

	  pub height: uint32_t, /* height of picture in mm */
	  pub width: uint32_t, /* width of picture in mm */

	  pub accel_flags: uint32_t, /* (OBSOLETE) see fb_info.flags */

	  /* Timing: All values in pixclocks, except pixclock (of course) */
	  pub pixclock: uint32_t, /* pixel clock in ps (pico seconds) */
	  pub left_margin: uint32_t, /* time from sync to picture	*/
	  pub right_margin: uint32_t, /* time from picture to sync	*/
	  pub upper_margin: uint32_t, /* time from sync to picture	*/
	  pub lower_margin: uint32_t,
	  pub hsync_len: uint32_t, /* length of horizontal sync */
	  pub vsync_len: uint32_t, /* length of vertical sync */
	  pub sync: uint32_t, /* see FB_SYNC_* */
	  pub vmode: uint32_t, /* see FB_VMODE_* */
	  pub rotate: uint32_t, /* angle we rotate counter clockwise */
	  pub colorspace: uint32_t, /* colorspace for FOURCC-based modes */
	  pub reserved: [uint32_t; 4], /* Reserved for future compatibility */
}

impl Default for fb_var_screeninfo {
    fn default() -> fb_var_screeninfo {
        fb_var_screeninfo {
            xres: 0,
	          yres: 0,
	          xres_virtual: 0,
	          yres_virtual: 0,
	          xoffset: 0,
	          yoffset: 0,
	          bits_per_pixel: 0,
	          grayscale: 0,
	          red: fb_bitfield::default(),
	          green: fb_bitfield::default(),
	          blue: fb_bitfield::default(),
	          transp: fb_bitfield::default(),
	          nonstd: 0,
	          activate: 0,
	          height: 0,
	          width: 0,
	          accel_flags: 0,
	          pixclock: 0,
	          left_margin: 0,
	          right_margin: 0,
	          upper_margin: 0,
	          lower_margin: 0,
	          hsync_len: 0,
	          vsync_len: 0,
	          sync: 0,
	          vmode: 0,
	          rotate: 0,
	          colorspace: 0,
	          reserved: [0,0,0,0],
        }
    }
}

/// Link external functions provided by the
/// kernel which didn't need shared libs.
extern {
    pub fn ioctl(fd: c_int, req: c_ulong, info: *mut fb_var_screeninfo) -> c_int;
}

/// Kernel font header struct with original
/// c code comments.
#[repr(C)]
#[derive(Debug)]
pub struct psf2_header {
    pub magic: [u8; 4],
    pub version: u32,
    pub headersize: u32, /* offset of bitmaps in file */
	  pub flags: u32,
	  pub glyph_count: u32, /* number of glyphs */
	  pub glyph_size: u32, /* number of bytes for each character */
	  pub glyph_height: u32, /* max dimensions of glyphs */
	  pub glyph_width: u32, /* charsize = height * ((width + 7) / 8) */
}

impl psf2_header {
    pub fn new(data: &[u8]) -> psf2_header {
        psf2_header {
            magic: [data[0], data[1], data[2], data[3]],
            version: from_le(&data, 4),
            headersize: from_le(&data, 8),
            flags: from_le(&data, 12),
            glyph_count: from_le(&data, 16),
            glyph_size: from_le(&data, 20),
            glyph_height: from_le(&data, 24),
            glyph_width: from_le(&data, 28),
        }
    }
}

/// Propper error handling is not implemented and could be done
/// by using a custom error type and a small abstraction layer
/// which passes the arguments to the c function and does
/// some error handling on the return values.
///
/// The main problem is to correctly cast the data structures
/// into c_void and and back without loosing informations.
fn main() {
    let fb = OpenOptions::new().read(true).append(true).open("/dev/fb0").unwrap();
    let fd = fb.into_raw_fd(); // open the framebuffer as raw filedescriptor

    // get basic infos about the framebuffer
    let mut info = fb_var_screeninfo::default();
    unsafe { ioctl(fd, FBIOGET_VSCREENINFO, &mut info) };

    // determine the framebuffer size and open an mmap region
    let len = 4 * info.xres * info.yres;
    let ptr: *mut u32 = unsafe { // create an mmap region with rust types
        libc::mmap(0 as *mut libc::c_void, len as usize,
                   libc::PROT_READ | libc::PROT_WRITE,
                   libc::MAP_SHARED, fd, 0) as *mut u32
    };
    // create a buffer on top of the pointer
    let buf = unsafe {
        std::slice::from_raw_parts_mut(ptr, len as usize)
    };

    // load the font file and decode the gz file.
    // the header is parsed out of a vector containing u8 data
    let font_file = File::open("/usr/share/kbd/consolefonts/Lat2-Terminus16.psfu.gz").unwrap();
    let mut decoder = GzDecoder::new(font_file);
    // the font vector contains all read informations
    let mut font = Vec::new();
    decoder.read_to_end(&mut font).ok();
    let header = psf2_header::new(&font);
    // one should check the magic array which is returned: [114,181,74,134]

    // create an glyph 2d array; 32 is header offset
    let mut glyphs: Vec<Vec<u8>> = vec![vec![0; header.glyph_size as usize]; header.glyph_count as usize];
    for nitem in 0..header.glyph_count {
        for size in 0..header.glyph_size {
            glyphs[nitem as usize][size as usize] = font[(32 + nitem * header.glyph_size + size) as usize];
        }
    }

    loop {
        // get local time as byte array
        let time = chrono::Local::now();
        let time_str = time.format("%H:%M").to_string();
        let time_bytes = time_str.as_bytes();
        let stride = header.glyph_size / header.glyph_height; // determine step size
        let mut i = 0;

        // refresh time every remaining second
        while i < (60 - time.time().second()) {
            let mut left = info.xres - header.glyph_width * (time_bytes.len() as u32);
            let bottom = header.glyph_height;

            // place lower border
            for x in left..info.xres {
                let index = bottom * info.xres + x;
                buf[index as usize] = FG
            }

            // left border
            for y in 0..bottom {
                let index = y * info.xres + left - 1;
                buf[index as usize] = FG;
            }

            // get time string as byte array
            for s in 0..time_bytes.len() {
                let mut glyph = &glyphs[time_bytes[s] as usize]; // use char code to index glyph

                for y in 0..header.glyph_height {
                    for x in 0..header.glyph_width { // print glyph on screen
                        let bits = glyph[(y * stride + x / 8) as usize];
                        let bit = bits >> (7 - x % 8) & 1;
                        let index = y * info.xres + left + x;
                        buf[index as usize] = if bit != 0 {FG} else {BG};
                    }
                }
                left += header.glyph_width;
            }
            std::thread::sleep(std::time::Duration::new(1,0));
            i += 1;
        }
    }
}
