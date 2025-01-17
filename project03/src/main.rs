use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

// use this if depending on local crate
use libsteg;


#[derive(Debug)]
pub enum StegError {
    BadDecode(String),
    BadEncode(String),
    BadError(String),
}//error in encoding or decoding

fn main() -> Result<(), StegError> {

    //prepare arguments and check if proper amount are provided
    let args: Vec<String> = env::args().collect();//arguments
    let thread_count = &args[1];//establish thread count

    //check proper arguments length
    if args.len()!=3 {
        eprintln!("You need to give 2 arguments");
        return Ok(())
    }
    
    match args.len() {
        3 => {
            
            let thread_count = thread_count.parse::<usize>().unwrap();//parse usize from thread count

            //path from second argument 
            let path_string = args[2].to_string();//read from this directory
            let path = Path::new(&path_string);// path from directory

            //vector for storing threads also mpsc channels
            let mut handles = vec![];
            let (sender, receiver) = mpsc::channel();

            //list of files
            let mut file_list: Vec<PathBuf> = Vec::new();
            
            let mut num_files = 0;//number of files
            //sorting for only ppm files
            for entry in fs::read_dir(path).expect("Path not found!") {
                let entry = entry.expect("Valid entry not found!");
                let path = entry.path();
                if path.extension().unwrap() == "ppm" {
                    file_list.push(path);
                    num_files+=1;
                }
                
            }

            //for each thread
            for i in 0..thread_count{

                let tx = sender.clone();//clone the send channel
                let mut job_list: Vec<String> = Vec::new();//initialize job list
                let decimal_length: f64 = file_list.len() as f64;
                let interval = (decimal_length/thread_count as f64).ceil();
                let interval: usize = interval as usize; //determine interval size
                let start =  interval*i; //determine start index for this threads jobs
                let mut last_index = start+interval; //set last index as interval distance from start
                if last_index>=file_list.len()-1 {last_index=file_list.len()-1;} // if last is greater than number of files, set to number of files -1
                
                let mut counter = start;//counter for which job to add

                //until the job list is of properlength(), add jobs
                while job_list.len()<interval{
                    if counter >= last_index {break;}//if counter is greater than index, dont' add
                    job_list.push(file_list[counter].clone().into_os_string().into_string().unwrap());//push the path to the job list
                    counter+=1;//increment
                }

                //spawn a thread
                let handle = thread::spawn(move||{

                    //while jobs are remaining
                    while job_list.len()!=0{

                        //create ppm file from job
                        let ppm = match libsteg::PPM::new(job_list[job_list.len()-1].clone()) {
                            Ok(ppm) => ppm,
                            Err(err) => panic!("Error: {:?}", err),
                         };
                        let decoded:String = decode_message(&ppm.pixels).unwrap();//decode the string
                        let payload = (job_list[job_list.len()-1].clone(),decoded);//create file and decoded message for payload
                        tx.send(payload).unwrap();//send the payload
                        job_list.pop();//take the job off the list
                    }
                });
                handles.push(handle);//add the thread to the group of handles
            }


            //vector of return values, for each file wait for decoded message and add to vector
            let mut returns = Vec::new();
            for _handle in 0..num_files-1 {
                let value = receiver.recv().unwrap();
                returns.push(value.clone());
            }

            //wait for each thread
            for thread in handles{thread.join().unwrap();}

            let mut final_string: String = String::from("");//output string
            returns.sort();//sort the returns by file name
            for r in returns{
                final_string = format!("{}{}",final_string,r.1);//format to add each message to output string
            }
            println!("{}\n",final_string);//print out output string
        }
        _ => println!("You need to give 2 or 4 arguments!"),
    }
    Ok(())
}

fn decode_message(pixels: &Vec<u8>) -> Result<String, StegError> {
    let mut message = String::from("");

    for mut bytes in pixels.chunks(8) {
        // eprintln!("chunk!");
        //i had to at this i know there is loss of data/extra data
        let base = [20,20,20,20,20,20,20,20];//space for printing
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