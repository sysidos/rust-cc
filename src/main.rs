/*
 */
#![feature(box_syntax)]
#![feature(fnbox)]	// For some pretty-printing hackery

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate utf8reader;
#[macro_use]
extern crate structopt;

mod parse;
mod types;
mod ast;


#[derive(StructOpt)]
struct Options
{
	#[structopt(parse(from_os_str))]
	input: ::std::path::PathBuf,


	#[structopt(short="I",parse(from_os_str))]
	include_dirs: Vec<::std::path::PathBuf>,
}

fn main()
{
	env_logger::init();

	// 1. Parse command line arguments
	let args: Options = ::structopt::StructOpt::from_args();
	
	let mut program = ::ast::Program::new();
	match ::parse::parse(&mut program, &args.input, args.include_dirs)
	{
	Err(e) => {
		panic!("Error parsing file: {:?}", e);
		},
	Ok(_) => {}
	}

	let stdout = ::std::io::stdout();
	::ast::pretty_print::write(stdout.lock(), &program);
}

// vim: ft=rust
