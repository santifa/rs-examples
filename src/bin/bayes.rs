//! A simple naive bayes classifier
//! with glass identification as example.
//!
//! ### Todo
//! At the moment this is taken as exactly as possible
//! from the python source without the magic of panda.
//! This results in complex functions and an unstructered data flow.
use std::io::{self, BufRead, BufReader};
use std::collections::HashMap;
use std::fs::File;
use std::string::String;
use std::fmt::Debug;
use std::f64::consts::PI;

const ATTR: &'static [&str] = &["RI", "Na", "Mg", "Al", "Si", "K", "Ca", "Ba", "Fe"];

/// Represent a data row
type Row = Vec<String>;
type Metric = HashMap<String, Vec<f64>>;

/// Represent test and training data
/// as well as the prior and mean variance information.
struct Data<'a> {
    header: Vec<&'a str>,
    test_data: Vec<Row>,
    train_data: Vec<Row>,
    /// Percentage of a class within the dataset P(C)
    prior: Metric,
    mean_variance: HashMap<String, Metric>,
}

// Remove later
impl<'a> Debug for Data<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Data {{\nHeader: {:?}\n", self.header)?;
        write!(f, "Testdata:\n{:?}\n\n", self.test_data)?;
        write!(f, "Traindata:\n{:?}\n\n", self.train_data)?;
        write!(f, "Prior:\n{:?}\n\n", self.prior)?;
        write!(f, "Mean_Variance:\n{:?}\n}}", self.mean_variance)
    }
}

impl<'a> Data<'a> {
    /// Read a csv table as input data and split the data in two pieces.
    fn read_data(f: &str) -> io::Result<(Vec<Row>, Vec<Row>)> {
        let lines = BufReader::new(File::open(f)?).lines();
        let mut rows: Vec<Vec<String>> = lines
            .map(|s| s.unwrap().split(',').map(|s| s.into()).collect())
            .collect();
        let half = rows.len() / 2;
        let test = rows.split_off(half);
        Ok((rows, test))
    }

    /// Calculate the P(C) probability which is the
    /// fraction of the sum of all classes and the length
    /// of the trainings data.
    fn prior(train: &[Row]) -> Metric {
        let mut counts = HashMap::new();
        // count unique values
        train.iter().for_each(|row| {
            if counts.contains_key(&row[row.len() - 1]) {
                let val = *counts.get_mut(&row[row.len() - 1]).unwrap();
                counts.insert(&row[row.len() - 1], val + 1);
            } else {
                counts.insert(&row[row.len() - 1], 1);
            }
        });

        let mut prior = HashMap::new();
        for (key, val) in counts.iter() {
            prior.insert((*key).clone(), vec![*val as f64 / train.len() as f64]);
        }
        prior
    }

    fn mean_variance(train: &[Row]) -> HashMap<String, Metric> {
        let mut classes = vec![];
        // save unique classes
        train.iter().for_each(|row| {
            if !classes.contains(&row[row.len() - 1]) {
                classes.push(row[row.len() - 1].clone());
            }
        });

        let mut mean_variance = HashMap::new();
        for class in classes {
            let filtered: Vec<&Row> = train
                .iter()
                .clone()
                .filter(|row| row[row.len() - 1] == class)
                .collect();
            let mut class_mean_variance = HashMap::new();

            for (i, attr) in ATTR.iter().enumerate() {
                let mut mean = filtered
                    .iter()
                    .fold(0., |acc, &x| acc + x[i].parse::<f64>().unwrap())
                    / filtered.len() as f64;

                let mut variance = filtered.iter().fold(0., |acc, &x| {
                    acc + (x[i].parse::<f64>().unwrap() - mean).powi(2)
                }) / filtered.len() as f64;
                class_mean_variance.insert((*attr).into(), vec![mean, variance]);
            }

            mean_variance.insert((*class).to_string(), class_mean_variance);
        }
        mean_variance
    }

    fn predict(&self) -> HashMap<String, String> {
        let mut predictions = HashMap::new();

        for row in &self.test_data {
            let mut result = HashMap::new();
            for (k, v) in &self.prior {
                let mut p = 0.;
                let mean_var = self.mean_variance.get(k).unwrap();
                for (i, attr) in ATTR.iter().enumerate() {
                    let prob = gaussian_probability_density(
                        row[i].parse::<f64>().unwrap(),
                        mean_var.get(*attr).unwrap()[0],
                        mean_var.get(*attr).unwrap()[1],
                    );
                    if prob > 0. {
                        p += prob.log10();
                    }
                }
                result.insert(k, p + v[0].log10());
            }
            let max = result
                .values()
                .fold(0., |acc, &x| if acc < x { x } else { acc });
            let pred: &str = result
                .keys()
                .find(|key| *result.get(*key).unwrap() == max)
                .unwrap();
            predictions.insert(row[0].clone(), pred.into());
        }
        predictions
    }

    /// Read the input data and calculate prior and mean-variance on
    /// the trainingsdata.
    fn new(header: Vec<&'a str>, f: &str) -> io::Result<Data<'a>> {
        let (train, test) = Data::read_data(&f)?;
        let prior = Data::prior(&train);
        let mean_variance = Data::mean_variance(&train);

        Ok(Data {
            header: header,
            test_data: test,
            train_data: train,
            prior: prior,
            mean_variance: mean_variance,
        })
    }
}

/// Calculate the probability for some input and the gaussian description (mean and variance).
fn gaussian_probability_density(x: f64, mean: f64, variance: f64) -> f64 {
    let exponent = ((x - mean).powi(2) / (2. * variance)).exp();
    1. / (2. * PI * variance).sqrt() * exponent
}

fn accuracy(test_set: &[Row], predictions: HashMap<String, String>) -> f64 {
    let mut correct = 0.;
    for row in test_set {
        if &row[row.len() - 1] == predictions.get(&row[0]).unwrap() {
            correct += 1.;
        }
    }
    (correct / test_set.len() as f64) * 100.
}

fn main() {
    let mut header = vec!["Num"];
    header.append(&mut ATTR.into());
    header.push("Class");
    let data = Data::new(header, "assets/glass.csv").unwrap();
    println!("Data: {:?}", data);
    let predictions = data.predict();
    println!("Predictions: {:?}", predictions);
    let accuracy = accuracy(&data.test_data, predictions);
    println!("Accuracy {} %", accuracy); // this should be 90 % something is missing
}
