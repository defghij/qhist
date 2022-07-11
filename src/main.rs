use std::io::{self, BufRead};
use std::path::PathBuf;
use std::fs::File;
use structopt::StructOpt;
use hdrhistogram::Histogram;

#[derive(Debug, StructOpt)]
#[structopt(name = "qhist", about = "description")]
struct Opt {
    /// Input File
    #[structopt(short,long)]
    input: PathBuf,

    /// Column
    #[structopt(short, long)]
    column: usize,

    /// Lower Percentile
    #[structopt(short, long, default_value="0")]
    lower: f64,

    /// upper Percentile
    #[structopt(short, long, default_value="100")]
    upper: f64,

    /// Max output
    #[structopt(short, long, default_value="100")]
    max_lines: usize,
}

fn main() {
    
    let opt: Opt = Opt::from_args();
    let column: usize = opt.column;
    let f_name: PathBuf = opt.input;
    let max_lines: usize = opt.max_lines;
    let upper_perc: f64= opt.upper;
    let lower_perc: f64 = opt.lower;
    if lower_perc > upper_perc {
        panic!("Lower percentile bound is greater than upper percentile bound");
    }

    let file:File = File::open(f_name).expect("file not found");

    let lines: Vec<u64> = io::BufReader::new(file)
        .lines()
        .map(|line| {
            let l = line.unwrap();
            let l: Vec<&str> = l.split(" ").collect();
            if l.len() <= column {
                panic!("Error! Given column does not exist in data for line:\n---\n{0}\n----", l.clone()[0]);
            }
            l[column].to_owned().parse::<u64>().expect("Value was not parsable to an integer")
    }).collect();

    let mut hist = Histogram::<u64>::new(3).expect("Unable to create histogram");

    for val in lines.iter() {
        hist.record(*val).expect("Value added to histogram is out of range");
    }
    let count: u64 = hist.len();
    let stddev = hist.stdev();
    let mean = hist.mean();

    println!("Samples: {0: >7}", count);
    println!("Max:  {0: >10.2}", hist.highest_equivalent(hist.value_at_percentile(100.)));
    println!("Min:  {0: >10.2}", hist.lowest_equivalent(hist.value_at_percentile(0.)));
    println!("Mean: {0: >10.2}", mean);
    println!("SD:   {0: >10.2}", stddev);
    println!("Percentile  count      value");

    let mut out: Vec<String> = Vec::new();
    for v in hist.iter_recorded() {
        if lower_perc < v.percentile() &&
           v.percentile() <= upper_perc && 
           v.count_at_value() != 0 {
            out.push(format!("{: >6.2} {: >10} {: >10}",
                v.percentile(),v.count_at_value(), v.value_iterated_to()));
        }
    } 

    out.reverse();
    let line_count = if out.len() < max_lines { out.len() } else { max_lines };
    for l in out[0..line_count].iter() {
        println!("{}", l);
    }

}
