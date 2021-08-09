use super::{arguments::_parse_args, io::_writeOutput, platforms::_platforms};
use ansi_term::Colour;
use futures::{stream::iter, StreamExt};
use reqwest::Client;
extern crate slack_hook;
use slack_hook::{Slack, PayloadBuilder};
use tokio;
use dotenv::dotenv;
use std::env;

#[tokio::main]
pub async fn _takeover(hosts: Vec<String>, threads: usize) -> std::io::Result<()> {
    
    dotenv().ok();

    let client = &Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let args = &_parse_args();
    let fetches = iter(hosts.into_iter().map(|url| async move {

        let channel: String = env::var("CHANNEL").expect("CHANNEL is not found");
        let webhook_url: String = env::var("WEBHOOKURL").expect("WEBHOOKURL is not found");

        match client.get(&url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    let platformName = _platforms(text);
                    match platformName == "None" {
                        true => {
                            if args.is_present("verbose") {
                                println!(
                                    "[{}] {}!",
                                    Colour::Blue.bold().paint("Not Vulnerable"),
                                    url
                                );
                            }
                        }
                        _ => {
                            println!(
                                "[{}]\t{} at {}!",
                                Colour::Red.bold().paint(&platformName),
                                Colour::White.bold().paint("Possible Sub-domain Takeover"),
                                url
                            );
                            let outputData = format!("[{}] {}\n", platformName, url);
                            if args.is_present("output") {
                                let fileName = args.value_of("output").unwrap();
                                _writeOutput(fileName.to_string(), outputData.to_string());
                            }
                            _send_to_slack(outputData, &channel, &webhook_url);
                        }
                    }
                }
                Err(_) => {
                    if args.is_present("verbose") {
                        println!(
                            "[{}]\tAn error occured for [{}].",
                            Colour::Green.bold().paint("ERROR"),
                            Colour::White.bold().paint(url)
                        )
                    }
                }
            },
            Err(_) => {
                if args.is_present("verbose") {
                    println!(
                        "[{}]\tAn error occured for [{}].",
                        Colour::Green.bold().paint("ERROR"),
                        Colour::White.bold().paint(url)
                    )
                }
            }
        }
    }))
    .buffer_unordered(threads)
    .collect::<Vec<()>>();
    fetches.await;
    /*
        In case you want to know how it works, here is a more simpler code explaining the overall workflow:
        let body = response.text().await?;
        if body.contains("<p><strong>There isn't a GitHub Pages site here.</strong></p>") {
            println!("GitHub Pages Sub-domain Takeover seems possible!");
        }
    */
    Ok(())
}

// send message to slack.
fn _send_to_slack(msg: String, channel: &str, webhook_url: &str) -> Result<(), slack_hook::Error> {
    
    let slack = Slack::new(webhook_url).unwrap();
    let p = PayloadBuilder::new()
        .text(msg)
        .channel(channel)
        .username("Subdomain Takerover Bot")
        .icon_emoji(":chart_with_upwards_trend:")
        .build()
        .unwrap();

    slack.send(&p)
}