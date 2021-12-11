use super::{Results, Test, KeyCode};
use std::time::SystemTime;
use std::io::Write;
/*
header
datetime    settings    results.cps.overall results.accuracy.overall
xx          yy          aa                  bb              


is is possible to implement linkedhashmap for results::..::per_key so chronological data could be read easier?

data    character (hopefully it is because it's linked) time elapsed since last      correct? (Some<T>)
index   words[index].event[index].key.code.(key=0)    results.cps.per_event[index] words[index].event[index].correct
1       aa                                             0 (first key)                             kk          
2       bbb                                             time                                       hh         
*/

pub struct Pumper {
    pub results_obj: Results,
    pub test_obj: Test,
    pub args: String,
} 

impl Pumper {
    fn gen_datetime(&self) -> u64 {
        let mut a: u64 = 0;
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {a = n.as_secs()},
            Err(_) => (),
        }
        return a;
    }

    fn gen_settings(&self) -> String {
        //TODO: do it
        String::new()
    }

    fn gen_csv_from_vec(&self, v: Vec<String>) -> String {
        let s = v.into_iter().map(|i| i.to_string()+", ").collect::<String>();
        return s;
    }

    // it can be done without these things by doing the string directly instead of making string to vector and make back to string
    //  but at least it is easier to manage for the time being
    fn unpack_header(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();

        // i am not sure if this is the rust way?
        ret.push(self.gen_datetime().to_string());
        ret.push(self.gen_settings().to_string());
        ret.push(self.results_obj.cps.overall.to_string());
        ret.push(self.results_obj.accuracy.overall.to_string());

        return ret;
    }

    fn unpack_data(&self) -> Vec<Vec<String>> {
        let mut ret: Vec<Vec<String>> = Vec::new();

        // is this the rust way? i dont even know, there are too many functions and i dont know how to use them
        // and again it's faster to do this directly instead of making string into vector then string again
        // but i think rust is fast so should it be fine?
        let mut true_index = 0;
        for index in 0..self.test_obj.words.len() {
            let curr_word = &self.test_obj.words[index];

            // i assumed the results.cps.per_event is logged after key is pressed
            for index2 in 0..curr_word.events.len() {
                let curr_char = &curr_word.events[index2];
                let mut curr_vec: Vec<String> = Vec::new();

                curr_vec.push((true_index).to_string());

                // i really dont know what i am doing
                match curr_char.key.code {
                    KeyCode::Char(character) => curr_vec.push(character.to_string()),
                    KeyCode::Backspace => curr_vec.push("backspace".to_string()),
                    KeyCode::Enter => curr_vec.push("enter".to_string()),
                    // KeyCode::Char(character) => curr_vec.push(character.to_string()),
                    _ => ()
                }

                // result.cps.per_event has one less index than the words[word] total index
                if true_index > 0 {
                    curr_vec.push(self.results_obj.cps.per_event[true_index-1].to_string());
                }
                else {
                    curr_vec.push("0".to_string());
                }

                // FIXME: if backspace is pressed, anything wrong typed for that word is marked correct, maybe it's not for this module but the other
                match curr_char.correct {
                    Some(i) => curr_vec.push(i.to_string()),
                    _ => ()
                }
                // curr_vec.push(curr_char.correct.to_string());

                ret.push(curr_vec);
                true_index+=1;
            }
        }
        return ret;
    }

    pub fn return_csv_format(&self) -> String {
        let mut ret: String = String::new();

        ret.push_str(&self.gen_csv_from_vec(self.unpack_header()));
        ret.push_str(&String::from("\n"));

        for s in self.unpack_data().iter() {
            // is this memory safe????
            ret.push_str(&self.gen_csv_from_vec(s.to_vec()));
            ret.push_str(&String::from("\n"));
        }

        ret.push_str("---\n");
        return ret;
    }
    
    // using dependancies is a bit too much
    pub fn write_csv(&self) {
        //TODO: make the file name more appealing
        let mut file = std::fs::OpenOptions::new().create(true).write(true).append(true).open("ttyper_record.csv").expect("Unable to open file");
        file.write_all(self.return_csv_format().as_bytes()).expect("write failed");
    }
}