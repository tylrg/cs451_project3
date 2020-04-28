use std::env;
use std::fs;
//use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
//use std::rc::Rc;
use std::str;
use std::sync::mpsc;
use std::thread;
//use std::time::Duration;

// use this if depending on local crate
use libsteg;

pub const MAX: usize = usize::max_value();

#[derive(Debug)]
pub enum StegError {
    BadDecode(String),
    BadEncode(String),
    BadError(String),
}

fn main() -> Result<(), StegError> {

    //prepare arguments and check if proper amount are provided
    let args: Vec<String> = env::args().collect();
    let thread_count = &args[1];
    if args.len()!=3 {
        eprintln!("You need to give 2 arguments");
        return Ok(())
    }

    //determine thread count
    

    match args.len() {
        3 => {
            
            
            let thread_count = thread_count.parse::<usize>().unwrap();
            //path from second argument 
            let path_string = args[2].to_string();//to second directory
            let path = Path::new(&path_string);
            println!("Input Path: {:?}", path);
            let current_dir = env::current_dir().expect("Current directory not found!");
            println!("Current Directory {:?}", current_dir);
            
            //vector for storing threads and return values from channel, also mpsc channels
            let mut handles = vec![];
            
            let (sender, receiver) = mpsc::channel();

            //number of files

            //list of files
            let mut file_list: Vec<PathBuf> = Vec::new();
            

            let mut num_files = 0;
            //sorting for only ppm files
            for entry in fs::read_dir(path).expect("Path not found!") {
                //print!("Found an entry\n");
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                if path.extension().unwrap() == "ppm" {
                    file_list.push(path);
                    num_files+=1;
                }
                //file_list.push(path);
                
            }
            //for value in &file_list {println!("PPM File: {:?}", value);}//printing the ppm values
            println!("Number of files: {}",num_files);

            let  f_l = &file_list.clone();

            let mut start = 0;

            for i in 0..thread_count{
                let tx = sender.clone();

                let mut job_list: Vec<(String)> = Vec::new();
                let decimal_length: f64 = file_list.len() as f64;
                let mut interval = (decimal_length/thread_count as f64).ceil();
                let mut interval: usize = interval as usize;
                start =  interval*i;
                //last_index = (last_index*thread_count)+1;
                let mut last_index = start+interval;
                if last_index>=file_list.len()-1 {last_index=file_list.len()-1;}
                //println!("Interval: {}",interval);
                println! ("Start and Last Index for Thread {}: {}-{}",i,start,last_index);
                
                
                

                let mut counter = start;
                while job_list.len()<interval{
                    if counter >= last_index {break;}
                    println!("Thread: {} is getting {}, responsible for {}/{}",i,counter,start,last_index-1);
                    job_list.push(file_list[counter].clone().into_os_string().into_string().unwrap());
                    counter+=1;
                    //if counter >= file_list.len(){break;}
                }

                let handle = thread::spawn(move||{
                    println!("Created thread: {} with {} jobs",i,job_list.len());
                    
                    while job_list.len()!=0{
                        let ppm = match libsteg::PPM::new(job_list[job_list.len()-1].clone()) {
                            Ok(ppm) => ppm,
                            Err(err) => panic!("Error: {:?}", err),
                         };
                        let decoded:String = decode_message(&ppm.pixels).unwrap();
                        //println!("Thread {} decoded: {} ",i,&decoded[0..10]);
                        let payload = (job_list[job_list.len()-1].clone(),decoded);
                        tx.send(payload).unwrap();
                        job_list.pop();
                    }
                });
                
                handles.push(handle);

            }
            let mut file_returns = Vec::new();
            let mut message_returns = Vec::new();
            let mut returns = Vec::new();
            for handle in 0..num_files-1 {
                //println!("Recieved file #{}",handle);
                let value = receiver.recv().unwrap();
                file_returns.push(value.0.clone());
                message_returns.push(value.1.clone());
                returns.push(value.clone());
            }

            
            for thread in handles{
                println!("Thread {:?} finished",thread);
                thread.join().unwrap();
            }
            println!("Returned items {}",returns.len());

            let mut final_string: String = String::from("");
            returns.sort();
            for r in returns{
                final_string = format!("{}{}",final_string,r.1);
            }
            println!("{}\n",final_string);        
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}

fn decode_message(pixels: &Vec<u8>) -> Result<String, StegError> {
    let mut message = String::from("");

    for mut bytes in pixels.chunks(8) {
        // eprintln!("chunk!");
        let base = [20,20,20,20,20,20,20,20];
        if bytes.len() < 8 {
            //panic!("There were less than 8 bytes in chunk");
            
            bytes= &base[0..base.len()];
        }

        let character = decode_character(bytes);

        if character > 127 {
            return Err(StegError::BadDecode(
                "Found non-ascii value in decoded character!".to_string(),
            ));
        }

        message.push(char::from(character));

        if char::from(character) == '\0' {
            // eprintln!("Found terminating null!");
            break;
        }
    }

    Ok(message)
}
fn decode_character(bytes: &[u8]) -> u8 {
    if bytes.len() != 8 {
        panic!("Tried to decode from less than 8 bytes!");
    }

    let mut character: u8 = 0b0000_0000;

    for (i, &byte) in bytes.iter().enumerate() {
        if lsb(byte) {
            match i {
                0 => character ^= 0b1000_0000,
                1 => character ^= 0b0100_0000,
                2 => character ^= 0b0010_0000,
                3 => character ^= 0b0001_0000,
                4 => character ^= 0b0000_1000,
                5 => character ^= 0b0000_0100,
                6 => character ^= 0b0000_0010,
                7 => character ^= 0b0000_0001,
                _ => panic!("uh oh!"),
            }
        }
    }

    character
}
fn lsb(byte: u8) -> bool {
    (0b0000_0001 & byte) == 1
}