use reqwest::header::CONTENT_TYPE;
use serde_json::{self, Value};
use std::error::Error;
use std::io::{self, Write};
use structopt::StructOpt;
use url::ParseError;
use url::Url;

#[derive(Debug, StructOpt)]
struct Cli {
    url: String,

    #[structopt(short)]
    data: Option<String>,

    #[structopt(long)]
    json: Option<String>,

    #[structopt(short = "X")]
    method: Option<reqwest::Method>,
}

// handle url errors
fn parse_url(url: &str) -> &str {
    let parsed = Url::parse(url);
    match parsed {
        Ok(_) => "",
        Err(e) => {
            match e {
                //invalid base
                ParseError::RelativeUrlWithoutBase
                | ParseError::RelativeUrlWithCannotBeABaseBase
                | ParseError::EmptyHost => "The URL does not have a valid base protocol.",

                // invalid address
                ParseError::InvalidIpv6Address => "The URL contains an invalid IPv6 address.",
                ParseError::InvalidIpv4Address => "The URL contains an invalid IPv4 address.",

                // invalid port
                ParseError::InvalidPort => "The URL contains an invalid port number.",

                //else
                _ => "",
            }
        }
    }
}

// sorted keys json format output
fn pretty_sorted_json(input: &str) -> Result<String, Box<dyn Error>> {
    let Ok(mut v) = serde_json::from_str::<Value>(input) else {
        println!("\nResponse body:");
        return Ok(input.to_string());
    };

    v.sort_all_objects();

    let out = serde_json::to_string_pretty(&v)?;
    println!("\nResponse body (JSON with sorted keys):");
    Ok(out)
}

// http body get
async fn body_get(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(url).await?;
    let body = body.error_for_status()?;
    let body = body.text().await?;
    Ok(body)
}

// get function
async fn http_get(url: &str) {
    println!("Requesting URL: {url}");
    println!("Method: GET");

    let err_msg = parse_url(url);
    if !err_msg.is_empty() {
        println!("Error: {err_msg}");
        return;
    }

    let body = body_get(url).await;
    match body {
        Ok(body) => {
            println!("Response body:\n{}", body);
        }
        Err(e) => {
            print!("Error: ");

            if e.is_connect() {
                print!(
                    "Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.\n"
                );
            } else {
                let code = e.status().unwrap().as_u16();
                print!("Request failed with status code: {}.", code);
            }
        }
    }
}

// post function
async fn http_post(args: &Cli) {
    println!("Requesting URL: {}", args.url);
    println!("Method: POST");
    let send_data: String;
    if args.json.is_some() {
        send_data = args.json.clone().unwrap();
        print!("JSON: {send_data}");
        io::stdout().flush().ok();
        let _v: Value =
            serde_json::from_str(args.json.clone().unwrap().as_str()).expect("Invalid JSON");
    } else {
        send_data = args.data.clone().unwrap();
        print!("Data: {send_data}");
    };

    let err_msg = parse_url(&args.url);
    if !err_msg.is_empty() {
        println!("Error: {err_msg}");
        return;
    }

    let body = body_post(&args, send_data).await;

    match body {
        Ok(body) => {
            let result = pretty_sorted_json(&body);
            match result {
                Ok(json) => println!("{}", json),
                Err(e) => println!("Error: {e}"),
            }
        }
        Err(e) => {
            print!("Error: ");

            if e.is_connect() {
                print!(
                    "Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.\n"
                );
            } else {
                let code = e.status().unwrap().as_u16();
                print!("Request failed with status code: {}.", code);
            }
        }
    }
}

// http body post
async fn body_post(args: &Cli, send_data: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut rb = client.post(args.url.as_str());

    rb = if args.json.is_some() {
        rb.header(CONTENT_TYPE, "application/json")
    } else {
        rb.header(CONTENT_TYPE, "application/x-www-form-urlencoded")
    };

    let resp = rb.body(send_data).send().await?.error_for_status()?;
    let body = resp.text().await?;
    Ok(body)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    // println!("{:?}", args);

    if args.json.is_some() || (args.data.is_some() && args.method.is_some()) {
        http_post(&args).await;
    } else {
        http_get(&args.url).await;
    }
    Ok(())
}
