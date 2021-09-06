use futures::executor::block_on;
use std::collections::HashMap;
use std::io::Write;
use reqwest::redirect::Policy;
use reqwest::{StatusCode, Response};
use std::process::exit;
use std::time::SystemTime;
use std::thread::Thread;

const samf_ticket_url: &'static str = "https://billettsalg.samfundet.no/pay";

fn main() {
    abc()
}

fn get_input(display_str: &str) -> String {
    let mut inp_line = String::new();

    print!("{}", display_str);
    std::io::stdout().flush();
    std::io::stdin().read_line(&mut inp_line);

    return inp_line;
}

fn poll_for_keys(base_url: &String) -> (String, String) {
    let mut member_key: Option<String> = Option::None;//"price_9302_count".to_string();
    let mut non_member_key: Option<String> = Option::None;//"price_9303_count".to_string();

    let mut max_itr = 100;
    let mut curr_itr = 0;
    let opt_resp: Option<reqwest::blocking::Response> = loop {
        print!("poll nmr {} for key \t", curr_itr);
        let response = reqwest::blocking::get(base_url).unwrap();
        match { response.status() } {
            StatusCode::NOT_FOUND => { std::thread::sleep(std::time::Duration::from_millis(100)) }
            _ => {
                println!("Hit!");
                break Option::Some(response);
            }
        }
        if max_itr < curr_itr {
            break Option::None;
        }
        curr_itr += 1;
        println!("Miss")
    };

    let response = opt_resp.unwrap();

    let resp_text = response.text().unwrap();
    let split_text = resp_text.split("\n");
    for line in split_text {
        if (line.contains("<tr data-price-group=\"")) {
            let target_num = line
                .split("data-price-group=\"")
                .skip(1)
                .next()
                .unwrap()
                .split("\"").next().unwrap();

            if member_key.is_none() {
                member_key = Option::Some(target_num.to_string())
            } else {
                non_member_key = Option::Some(target_num.to_string())
            }
            // println!("{}", target_num)
        }
    }
    // println!("{}", response.status());
    // println!("{}", response.text().unwrap());

    return (format!("price_{}_count", member_key.unwrap()), format!("price_{}_count", non_member_key.unwrap()));
    // return (( member_key.unwrap()), non_member_key.unwrap());
}


fn abc() {
    let mut form_vals: HashMap<String, String> = HashMap::new();
    // let mut email: String = "moxiv51007@rebation.com".to_string();//get_input("Ticket email: ");
    // let mut card_nmr: String = "4925000000000004".to_string();// get_input("card nmr: ");
    // let mut exp_m: String = "09".to_string();//get_input("card exp month (write 01 not 1 if 1 in 01/83): ");
    // let mut exp_y: String = "21".to_string();//get_input("card exp year :");
    // let mut cvc2: String = "123".to_string();//get_input("cvc 2:");

    let mut email: String = get_input("Ticket email: ");
    let mut card_nmr: String = get_input("card nmr: ");
    let mut exp_m: String = get_input("card exp month (write 01 not 1 if 1 in 01/83): ");
    let mut exp_y: String = get_input("card exp year :");
    let mut cvc2: String = get_input("cvc 2:");

    form_vals.insert("wheelchair-total".to_string(), "0".to_string());
    form_vals.insert("ticket-type".to_string(), "on".to_string());
    form_vals.insert("email".to_string(), email);
    form_vals.insert("ccno".to_string(), card_nmr);
    form_vals.insert("exp_month".to_string(), exp_m);
    form_vals.insert("exp_year".to_string(), exp_y);
    form_vals.insert("cvc2".to_string(), cvc2);

    println!("\ncopy the target url in this one the target url shold look somthing like this");
    println!("https://www.uka.no/program/658-bli-stor-pa-some-mmaria-stavang/758");
    println!("So that is: https://www.uka.no/program/<number somthing>-<name somthing>/<som other number>");
    let target_url = get_input("target url: ");//"https://www.uka.no/program/658-bli-stor-pa-some-mmaria-stavang/758".to_string();
    // let target_url = "https://www.uka.no/program/658-bli-stor-pa-some-mmaria-stavang/758".to_string();

    println!("\nGo here to find the epoch time https://www.epochconverter.com/");
    open::that("https://www.epochconverter.com/");
    let target_time_string = get_input("target time in unix epoc:").trim().to_string();
    let target_epoch_time = target_time_string.parse::<u64>().unwrap();
    let wait_time = target_epoch_time - std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let sleep_time = std::time::Duration::from_secs(wait_time - 2);

    println!("The system is primed and will wait for the launch point in {} seconds", sleep_time.as_secs());
    std::thread::sleep(sleep_time);

    println!("Starting preemptive polling for keys");

    let (member_key, non_member_key) = poll_for_keys(&(target_url + "/billetter"));

    println!("Member key value \t{}", member_key);
    println!("Non member key value \t{}", non_member_key);

    form_vals.insert(member_key, "1".to_string());
    form_vals.insert(non_member_key, "0".to_string());

    let no_redirect_client = reqwest::blocking::ClientBuilder::new().redirect(Policy::none()).build().unwrap();

    println!("posting form to samf's shitty slow website");
    let resp = no_redirect_client.post(samf_ticket_url).form(&form_vals).send().unwrap();
    match resp.status() {
        StatusCode::SEE_OTHER => {
            println!("Success maybe. this is the part i hav not tested so idk realy")
        }
        _ => {}
    }

    resp.headers().iter().for_each(|(name, value)| {
        // println!("{}-{}", name, value.to_str().unwrap());
        if name == "location" {
            println!("\nlocation hit:\n{}\n", value.to_str().unwrap());
            open::that(value.to_str().unwrap());
        }
    });
}
