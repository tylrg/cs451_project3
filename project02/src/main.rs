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
use std::sync::{Arc, Mutex};
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
    // if args.len()!=3 && args.len()!=5 {
    //     eprintln!("You need to give 2 or 4 arguments!");
    //     return Ok(());
    // }else{
    //     let thread_count = &args[1];
    //     //let thread_count = thread_count.parse::<usize>().unwrap();
    //     println!("THREAD COUNT: {}",thread_count);
    // }

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
            let mut num_files = 0;
            //increment for each file in directory
            //for _entry in fs::read_dir(path).expect("Path not found!") {num_files = num_files + 1;}

            //list of files
            let mut file_list: Vec<PathBuf> = Vec::new();
            

            //return Ok(());

            //shadowing the number of files

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
                let mut last_index = (decimal_length/thread_count as f64).ceil();
                let mut last_index: usize = last_index as usize;
                start =  last_index*i;
                //println!("Last value: {}",last_index);
                //println!("Start: {}/Last: {}",start,last_index);
                //let mut last_index = (file_list.len()/thread_count)*(i+1);
                // if last_index>file_list.len(){
                //     last_index=file_list.len();
                // }

                let mut counter = start;
                while job_list.len()<last_index{
                    if counter == file_list.len(){break;}
                    println!("Thread: {} has {}",i,counter);
                    job_list.push(file_list[counter].clone().into_os_string().into_string().unwrap());
                    counter+=1;
                }



                let handle = thread::spawn(move||{
                    println!("Created thread: {} with {} jobs",i,job_list.len());
                    
                    while job_list.len()!=0{
                        let ppm = match libsteg::PPM::new(job_list[job_list.len()-1].clone()) {
                            Ok(ppm) => ppm,
                            Err(err) => panic!("Error: {:?}", err),
                         };
                        let decoded:String = decode_message(&ppm.pixels).unwrap();
                        let payload = (job_list[job_list.len()-1].clone(),decoded);
                        tx.send(payload).unwrap();
                        job_list.pop();
                    }
                });
                
                handles.push(handle);

                //start = last_index+1;
            }
            let mut file_returns = Vec::new();
            let mut message_returns = Vec::new();
            let mut returns = Vec::new();
            for _handle in 0..num_files {
                let value = receiver.recv().unwrap();
                file_returns.push(value.0.clone());
                message_returns.push(value.1.clone());
                returns.push(value.clone());
            }

            
            for thread in handles{thread.join().unwrap();}
            //println!("Returned items {}",returns.len());

            let mut final_string: String = String::from("");
            returns.sort();
            for r in returns{
                final_string = format!("{}{}",final_string,r.1);
            }
            println!("{}\n",final_string);        
        }
        5 => {
            let thread_count = thread_count.parse::<usize>().unwrap();
            //cargo run <numThreads> <message file> <ppm directory> <output directory>

            //print out the current directory
            //let current_dir = env::current_dir().expect("Current directory not found!");
            //println!("Current Directory {:?}", current_dir);

            let mut handles = vec![];

            //let the message be the input from a file //ARGS 2
            let mut message = match fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(err) => return Err(StegError::BadEncode(err.to_string())),
            };
            //println!("Total bytes of message: {}", message.capacity());

            let end = vec![0];
            let end = str::from_utf8(&end).unwrap();
            let end:String = String::from(end);
            let end =  end.chars();
            //println!("{}",end.clone().next().unwrap());
            message.push(end.clone().next().unwrap());

            let message = message.as_bytes();
            
            //println!("Message as bytes: {:?}",message);

            //get path from input file
            let path_string = args[3].to_string(); //ARGS 3 input directory
            let path = Path::new(&path_string);
            //println!("Path provided {:?}",path);

            let mut total_size:usize = 0;
            
            

            let mut file_list: Vec<String> = Vec::new();

            for entry in fs::read_dir(path).expect("Path not found!") {
                //println!("Found an entry {:?}",entry);
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                
                if path.extension().unwrap() != "ppm" {continue;}
                let path = path.into_os_string().into_string().unwrap();
                let path_str = path.clone();

                file_list.push(path_str);
                
                let ppm = match libsteg::PPM::new(path) {
                    Ok(ppm) => ppm,
                    Err(err) => panic!("Error: {:?}", err),
                };
                total_size+=ppm.pixels.len();
                //print!(" Pixels: {}\n",ppm.pixels.len());

                //comparison
                
            }
            //println!("Total Size: {} Available Size: {}",total_size,total_size/8);
            let total_size=total_size/8;
            


            if message.len() > total_size{return Ok(());}
            //for e in file_list.clone() {println!("File: {}",e);}

            
            //let largest_file = get_biggest(&file_list);
            let largest_file = file_list[0].clone();
            //println!("Largest File {}",largest_file.clone());
            let file_size = pixel_size(largest_file.clone());
            let output_dir = String::from(&args[4]);

        
            // job;
            // for loop in threadcount{
            //     job =;
            //     spawn thread
            // }
            //let mut Vec<Vec<(String, String)>> taco;

            
            
            let mut index = 0;
            //let message_parts_count = total_size/thread_count;
            let mut start_slice = 0;
            let mut end_slice = 0;
            
            
            let mut jobs: Vec<(String,String)> = Vec::new();
            //message //filename


            
            while start_slice<message.len() {
                //let file_to_use;


                let min = message.len();
                end_slice = end_slice+file_size/8;
                if end_slice>min {end_slice=min;}

                //start_slice = start_slice/8;
                //end_slice= end_slice/8;
                
                //println!("Start of slice: {} and end of slice: {}",start_slice,end_slice);



                let message_fragment = &message[start_slice..end_slice];
                let mut str_builder: Vec<u8> = Vec::new();
                for element in message_fragment.iter() {str_builder.push(*element);}
                let assembled = String::from_utf8(str_builder).unwrap();
                //println!("Adding : {}",assembled);

                let write_name = pad_zeros_for_file(index);
                let write_name=format!("{}/{}",output_dir,write_name);
                let job_value = (assembled,write_name);
                jobs.push(job_value);
                index+=1;
                //if index == message.len(){index=0;}

                
                start_slice=end_slice;
            }

            //println!("Jobs: {}", jobs.len());
            // for job in jobs.clone(){
            //     //println!("{:?}",job.1);
            // }      
            
            
            let mut start = 0;
            //let last_index = 0;
            for i in 0..thread_count{
                           
                //let pair = (1, true);
                //let job = i;
                
                let mut job_list: Vec<(String,String)> = Vec::new();

                let mut last_index = (thread_count*i)+thread_count+1;
                last_index = last_index *8;

                if last_index > jobs.len()-1{
                    last_index=jobs.len()-1;
                }
                
                if i != 0{
                    start = thread_count*(i);
                    start+=1;
                }

                
                
                for k in start..last_index+1{
                    job_list.push(jobs[k].clone());
                }

                //println!("LOOP {} Start: {} End: {}",i+1,start,last_index);


                let out = largest_file.clone();
                let handle = thread::spawn(move || {
                    //println!("Spawned thread: #{} Number of Jobs: {}",j,job_list.len());
                    while job_list.len() !=0 {
                        println!("Thread #{} :Writing a file {:?} Length of message: {}",i,job_list[job_list.len()-1].1,job_list[job_list.len()-1].0.len()-1);
                        
                        writeout(job_list[job_list.len()-1].0.clone(),out.clone(),job_list[job_list.len()-1].1.clone()).expect("What went wrong?");    

                        job_list.pop();                    
                    }
                });
                handles.push(handle);

                start = last_index+1;
            }
            for thread in handles{thread.join().unwrap();}
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}


fn encode_message(message: &str, ppm: &libsteg::PPM) -> Result<Vec<u8>, StegError> {
    let mut encoded = vec![0u8; 0];
    //println!("GOT INTO ENDCODE! Message len");
    // loop through each character in the message
    // for each character, pull 8 bytes out of the file
    // encode those 8 bytes to hide the character in the message
    // add those 8 bytes to the enocded return value
    // add a trailing \0 after all character encoded
    // output the remainder of the original file

    let mut start_index = 0;
    //println!("Message chars {:?}",message.chars());
    for c in message.chars() {
        encoded.extend(&encode_character(
            c,
            &ppm.pixels[start_index..start_index + 8],
        ));
        start_index += 8;
        //println!("{}",start_index);
    }
    
    // we need to add a null character to signify end of
    // message in this encoded image
    // encoded.extend(&encode_character(
    //     '\0',
    //     &ppm.pixels[start_index..start_index + 8],
    // ));

    // start_index += 8;

    // spit out remainder of ppm pixel data.
    encoded.extend(&ppm.pixels[start_index..]);
    
    Ok(encoded)
}
fn encode_character(c: char, bytes: &[u8]) -> [u8; 8] {
    let c = c as u8;

    let mut ret = [0u8; 8];

    for i in 0..bytes.len() {
        if bit_set_at(c, i) {
            ret[i] = bytes[i] | 00000_0001;
        } else {
            ret[i] = bytes[i] & 0b1111_1110;
        }
    }

    ret
}
fn bit_set_at(c: u8, position: usize) -> bool {
    bit_at(c, position) == 1
}
fn bit_at(c: u8, position: usize) -> u8 {
    (c >> (7 - position)) & 0b0000_0001
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

fn writeout(message_file: String,ppm_name: String,output_file_name: String) -> std::io::Result<()> {
    //let mut file = File::create(output_file_name)?;
    
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };

    let mut buffer = File::create(output_file_name).expect("Could not create file");
   
    match encode_message(&message_file, &ppm) {
                Ok(bytes) => {
                    // first write magic number
                     buffer
                         .write(&ppm.header.magic_number)
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
                    //println!("{}",&ppm.header.magic_number.to_string());
                    //println!("P6");
                     buffer
                         .write(&"\n".as_bytes())
                         .expect("FAILED TO WRITE MAGIC NUMBER TO STDOUT");
                    //print!("{:?}",&"\n".as_bytes());
                    // then the width
                    buffer
                         .write(ppm.header.width.to_string().as_bytes())
                         .expect("FAILED TO WRITE WIDTH TO STDOUT");
                    //print!("{}",ppm.header.width.to_string());
                    buffer
                        .write(&" ".as_bytes())
                        .expect("FAILED TO WRITE WIDTH TO STDOUT");
                    //print!(" ");
                    // then the height
                    buffer
                        .write(ppm.header.height.to_string().as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    //print!("{}",ppm.header.height.to_string());
                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE HEIGHT TO STDOUT");
                    //print!("\n");
                    // then the color value
                    buffer
                        .write(ppm.header.max_color_value.to_string().as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
                    //println!("{}",ppm.header.max_color_value.to_string());
                    buffer
                        .write(&"\n".as_bytes())
                        .expect("FAILED TO WRITE MAX COLOR VALUE TO STDOUT");
                    //print!("{:?}",&"\n".as_bytes());

                    // then the encoded byets
                    buffer
                        .write(&bytes)
                        .expect("FAILED TO WRITE ENCODED BYTES TO STDOUT");
                    
                }
                Err(err) => match err {
                    StegError::BadEncode(s) => panic!(s),
                    _ => panic!("RECEIVED AN UNEXPECTED ERROR WHEN TRYING TO ENCODE MESSAGE"),
                },
            }
    Ok(())
}

fn pad_zeros_for_file(index: usize) -> String{
    let mut ret_val:String = index.to_string();
    while ret_val.len() != 5{
        ret_val = format!("0{}",ret_val);
    }
    ret_val=format!("{}.ppm",ret_val);
    return ret_val;
}
fn pixel_size(ppm_name: String)-> usize{
    let ppm = match libsteg::PPM::new(ppm_name) {
                Ok(ppm) => ppm,
                Err(err) => panic!("Error: {:?}", err),
    };
    return ppm.pixels.len();
}