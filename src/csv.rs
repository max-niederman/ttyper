use super::test::{results};


use std::time::SystemTime;

/*
header
datetime    settings    results.cps.overall results.accuracy.overall
xx          yy          aa                  bb              


is is possible to implement linkedhashmap for results::..::per_key so chronological data could be read easier?

data    character (hopefully it is because it's linked) time elapsed since last      correct? (Some<T>)
index   words[index].event[index].key.code.(key=0)    results.cps.per_event[index] words[index].event[index].correct
1       aa                                              yy                             kk                   
*/

pub struct Pumper {
    result_obj: results::Results,
    // test_obj: results::Test,
    args: String,
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

    fn gen_csv_from_vec(v: Vec<String>) -> String {
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
        ret.push(self.result_obj.cps.overall.to_string());
        ret.push(self.result_obj.accuracy.overall.to_string());

        return ret;
    }

    // fn unpack_data(&self) -> Vec<String> {
    //     for 
    // }
}