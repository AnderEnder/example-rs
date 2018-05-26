#[macro_use]
extern crate structopt;

#[macro_use]
extern crate failure;

extern crate futures;
extern crate indicatif;
extern crate rusoto_core;
extern crate rusoto_s3;

use failure::{err_msg, Error};
use futures::Future;
use futures::stream::Stream;
use indicatif::{ProgressBar, ProgressStyle};
use rusoto_core::Region;
use rusoto_s3::{GetObjectRequest, Object, S3, S3Client};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use structopt::StructOpt;
use structopt::clap::AppSettings;

type Result<T> = ::std::result::Result<T, Error>;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "example", about = "download s3 file",
            raw(global_settings = "&[AppSettings::ColoredHelp, AppSettings::NeedsLongHelp, AppSettings::NeedsSubcommandHelp]"))]
pub struct Opt {
    #[structopt(name = "bucket")]
    bucket: String,
    #[structopt(name = "src")]
    src: String,
    #[structopt(name = "dest")]
    dest: String,
}

fn main() -> Result<()> {
    let opts = Opt::from_args();
    let region = Region::default();
    let client = S3Client::simple(region);

    let key = opts.src.clone();
    let target = opts.dest.clone();
    let bucket = opts.bucket.clone();

    let request = GetObjectRequest {
        bucket: bucket.to_owned(),
        key: key.to_owned(),
        ..Default::default()
    };

    // let size = (*object.size.as_ref().unwrap()) as u64;
    let file_path = Path::new(&target).join(key);
    let dir_path = file_path.parent().ok_or(err_msg("parse parent"))?;

    if file_path.exists() {
        return Err(err_msg("file is already present"));
    }

    let mut count: u64 = 0;

    let pb = ProgressBar::new(0);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    let result = client.get_object(&request).sync()?;

    let mut stream = result
        .body
        .ok_or(err_msg("cannot fetch body from s3 response"))?;

    fs::create_dir_all(&dir_path)?;
    let mut output = File::create(&file_path)?;

    let _r = stream
        .for_each(|buf| {
            output.write(&buf)?;
            count = count + (buf.len() as u64);
            pb.set_position(count);
            Ok(())
        })
        .wait();

    Ok(())
}
