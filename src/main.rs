use std::env;
use std::error::Error;
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
            println!("Response body:\n{}", body);
        }
        Err(e) => {
            print!("Error: ");

            if e.is_connect() {
                print!(
                    "Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.\n"
                );
            } else if let code = e.status().unwrap().as_u16() {
                print!("Request failed with status code: {}.", code);
            }
        }
    }
}

async fn http_post(args: &Cli) {
    println!("Requesting Url: {}", args.url);
    println!("Method: POST");
    let send_data = if args.json.is_some() {
        args.json.as_ref().unwrap()
    } else {
        args.data.as_ref().unwrap()
    };

    println!("Data: {}", send_data);

    let err_msg = parse_url(&args.url);
    if !err_msg.is_empty() {
        println!("Error: {err_msg}");
        return;
    }
    let body = body_post(&args, send_data.clone()).await;

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
            } else if let code = e.status().unwrap().as_u16() {
                print!("Request failed with status code: {}.", code);
            }
        }
    }
}

async fn body_post(args: &Cli, send_data: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let body = client
        .post(args.url.as_str())
        .body(send_data)
        .send()
        .await?;
    let body = body.error_for_status()?;
    let body = body.text().await?;
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
