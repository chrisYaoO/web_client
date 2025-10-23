use reqwest::header::CONTENT_TYPE;
use serde_hjson;
use serde_json::{self, Map, Value, de};
use std::error::Error;
use structopt::StructOpt;
use url::ParseError;
use url::Url;

#[derive(Debug, StructOpt)]
struct Cli {
    url: String,

    #[structopt(short)]
    data: Option<String>,

    #[structopt(long, parse(try_from_str = parse_json))]
    json: Option<Value>,

    #[structopt(short = "X")]
    method: Option<reqwest::Method>,
}

fn parse_json(s: &str) -> Result<Value, String> {
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn parse_url(url: &str) -> &str {
    let parsed = Url::parse(url);
    match parsed {
        Ok(url) => "",
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

async fn body_get(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(url).await?;
    let body = body.error_for_status()?;
    let body = body.text().await?;
    Ok(body)
    // match body {
    //     Ok(body) => match body.error_for_status() {
    //         Ok(body) => body.text().await,
    //         Err(e) => Err(e),
    //     },
    //     Err(e) => Err(e),
    // }
}

async fn http_get(url: &str) {
    println!("Requesting Url: {url}");
    println!("Method: GET");

    let err_msg = parse_url(url);
    if !err_msg.is_empty() {
        println!("Error: {err_msg}");
        return;
    }

    let body = body_get(url).await;
    match body {
        Ok(body) => {
            print!("Response body:\n{}", body);
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

pub fn pretty_sorted_json(input: &str) -> Result<String, Box<dyn Error>> {
    let Ok(mut v) = serde_json::from_str::<Value>(input) else {
        println!("Response body:");
        return Ok(input.to_string());
    };

    v.sort_all_objects();

    let out = serde_json::to_string_pretty(&v)?;
    println!("Response body (JSON with sorted keys):");
    Ok(out)
}

fn json_sorted_pretty(v: &Value) -> String {
    // 按键排序 + pretty
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    String::from_utf8(buf).unwrap()
}

async fn http_post(args: &Cli) {
    println!("Requesting Url: {}", args.url);
    println!("Method: POST");
    let mut send_data = String::new();
    if args.json.is_some() {

        println!("Response body (JSON with sorted keys):\n");
    } else {
        send_data = args.data.clone().unwrap();
        println!("Data:{}", send_data);
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
    println!("{:?}", args);

    if args.json.is_some() || (args.data.is_some() && args.method.is_some()) {
        http_post(&args).await;
    } else {
        http_get(&args.url).await;
    }
    Ok(())
}
