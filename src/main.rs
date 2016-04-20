mod um;

use std::fs::File;
use std::io::{Read, Write};
use std::env;
use std::error::Error;

fn barf(msg: &str) -> !
{
    writeln!(&mut std::io::stderr(), "{}", msg).unwrap();
    std::process::exit(-1);
}

/* TODO: Make this better */
fn read_program_from_file<T>(mut file: T) -> std::io::Result<Vec<u32>> where T: Read
{
    let mut prog = Vec::new();

    loop
    {
        let mut platter;
        let mut buf = [0u8; 4];

        let len = try!(file.read(&mut buf));
        if len == 0
        {
            break;
        }
        else if len != 4
        {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "read_program_from_file: file not aligned"));
        }

        platter = buf[3] as u32;
        platter |= (buf[2] as u32) << 8;
        platter |= (buf[1] as u32) << 16;
        platter |= (buf[0] as u32) << 24;

        prog.push(platter);
    }

    Ok(prog)
}

fn main() {
    let mut args = env::args();
    let progname = args.next().unwrap_or(String::from("um"));
    let filename;
    let prog_file;

    if let Some(fname) = args.next()
    {
        filename = fname;
    }
    else
    {
        writeln!(&mut std::io::stderr(), "Usage: {} FILE", progname).unwrap();
        std::process::exit(-1);
    }

    match File::open(filename.clone())
    {
        Ok(f) =>
        {
            prog_file = f;
        },
        Err(e) =>
        {
            writeln!(&mut std::io::stderr(), "{}: {}: {}", progname, filename, e.description()).unwrap();
            std::process::exit(-1);
        },
    }

    let prog;
    match read_program_from_file(prog_file)
    {
        Ok(p) => prog = p,
        Err(e) => barf(e.description()),
    }

    let mut um = um::Um::new(prog);

    println!("Running program {}", filename);

    loop
    {
        match um.next_op()
        {
            Ok(running) =>
            {
                if !running
                {
                    break;
                }
            },
            Err(e) =>
            {
                println!("Error: {}", e.description());
                println!("UM State:");
                um.print_state();
                break;
            }
        }
    }
    println!("Program finished");
}
