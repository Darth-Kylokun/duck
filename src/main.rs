use std::env;
use hyper::{
    Client,
    StatusCode
};
use tokio::{
    io::{
        BufReader,
        BufWriter,
        AsyncBufReadExt,
        AsyncWriteExt
    },
    fs::File
};
use hyper_tls::HttpsConnector;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type TokioResult<T> = tokio::io::Result<T>;
type HyperClient = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Please give args of: <INPUT_FILE> <OUTPUT_FILE>");
        return Ok(());
    }

    let mut to_read = File::open(&args[1]).await?;
    let mut to_write = File::create(&args[2]).await?;

    let https = HttpsConnector::new();
    let cli = Client::builder().build::<_, hyper::Body>(https);
    let mut past_urls: Vec<String> = Vec::new();

    let mut i = 1;

    loop {
        tokio::select! {
            _ = async {} => {
                let urls = read_in_file(&mut to_read).await?;
                    if past_urls.len() != urls.len() {
                    println!("");
                    let new_urls: Vec<String> = urls.into_iter().filter(|url| past_urls.contains(url) != true).collect();

                    for url in new_urls.iter() {
                        let status = test_url(&cli, &url).await?;
                        if status == StatusCode::OK {
                            write_to_file(&mut to_write, url).await?;
                        }
                        println!("Status of url {} is: {}", i, status);
                        i += 1;
                    }

                    past_urls = new_urls;
                } else {
                    print!("\rWaiting");
                }
            }
        }
    }
}

async fn test_url<'r>(cli: &HyperClient, url: &'r str) -> Result<StatusCode> {
    let uri = match url.parse::<hyper::Uri>() {
        Ok(u) => u,
        Err(_) => return Ok(StatusCode::BAD_REQUEST)
    };
    let resp = cli.get(uri).await?;

    Ok(resp.status())
}

async fn read_in_file(f: &mut File) -> TokioResult<Vec<String>> {
    let reader = BufReader::new(f);
    let mut res: Vec<String> = Vec::new();

    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        res.push(line);
    }

    Ok(res)
}

async fn write_to_file<'r>(f: &mut File, to_write: &'r str) -> TokioResult<()> {
    let mut writer = BufWriter::new(f);

    writer.write(format!("{}\n", to_write).as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}