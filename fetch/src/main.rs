use crate::ignore::Ignore;
use anyhow::{bail, Context, Result};
use log::*;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

mod ignore;

#[derive(Serialize, Deserialize)]
struct EnemyInfo {
    user: String,
    lgtm: u32,
    ext: String,
}

#[derive(Serialize, Deserialize)]
struct EnemyList {
    enemies: Vec<EnemyInfo>,
}

#[derive(StructOpt)]
struct Opt {
    /// Url to the source page
    #[structopt(
        short = "s",
        long = "source-url",
        default_value = "https://qiita.com/torifukukaiou/items/c8361231cdc56e493245"
    )]
    url: String,
    /// Path to the temporary directory to store downloaded images
    #[structopt(short = "d", long = "download-dir", default_value = "downloaded")]
    downloaded_dir: PathBuf,
    /// Path to the temporary directory to store processed images
    #[structopt(short = "r", long = "process-dir", default_value = "processed")]
    processed_dir: PathBuf,
    /// Width of the enemy texture
    #[structopt(short = "w", long = "width", default_value = "32")]
    width: u32,
    /// Height of the enemy texture
    #[structopt(short = "h", long = "height", default_value = "32")]
    height: u32,
    /// Run image processing only
    #[structopt(short = "p", long = "process-only")]
    process_only: bool,
    #[structopt(short = "i", long = "ignore-list")]
    ignore_list: Option<PathBuf>,
    /// Path to the output texture file
    #[structopt(name = "output_texture")]
    output_texture: PathBuf,
    /// Path to the output info file
    #[structopt(name = "output_info")]
    output_info: PathBuf,
}

async fn fetch_page(url: &str) -> Result<String> {
    info!("Fetching page from {}", url);
    reqwest::get(url)
        .await
        .with_context(|| format!("cannot get the page: {}", url))?
        .text()
        .await
        .with_context(|| format!("cannot get the body of the page: {}", url))
}

async fn fetch_image(url: &str) -> Result<(Vec<u8>, String)> {
    info!("Fetching image from {}", url);
    let resp = reqwest::get(url)
        .await
        .with_context(|| format!("cannot get the image: {}", url))?;

    let ctype: &str = resp
        .headers()
        .get("content-type")
        .with_context(|| format!("cannot determine content type: {}", url))?
        .to_str()
        .with_context(|| format!("value in `content-type` is not a string: {}", url))?;

    let ext = match ctype {
        "image/gif" => "gif",
        "image/png" => "png",
        "image/jpg" | "image/jpeg" => "jpg",
        e => bail!("unsupported content type: {}", e),
    };

    let content = resp
        .bytes()
        .await
        .with_context(|| format!("cannot get the image body: {}", url))?;

    Ok((content.to_vec(), ext.into()))
}

fn load_ignore_list(opt: &Opt) -> Result<Ignore> {
    match &opt.ignore_list {
        Some(path) => Ignore::load(&path),
        None => Ok(Ignore::default()),
    }
}

async fn download_images(opt: &Opt) -> Result<()> {
    let ignore = load_ignore_list(opt)?;

    tokio::fs::create_dir_all(&opt.downloaded_dir).await?;

    let body = fetch_page(&opt.url).await?;

    let doc = Html::parse_document(&body);
    let trsel = Selector::parse("tr").unwrap();
    let tdsel = Selector::parse("td").unwrap();
    let asel = Selector::parse("a").unwrap();
    let imgsel = Selector::parse("img").unwrap();

    let mut enemies = EnemyList { enemies: vec![] };

    for e in doc.select(&trsel) {
        info!("{}", e.inner_html());
        let e: Vec<_> = e.select(&tdsel).collect();
        if e.len() != 5 {
            warn!("the table size != 5, skipping: {:?}", e);
            continue;
        }

        let a = e[1].select(&asel).nth(1).context("no 2nd a tag")?;
        let user = a.value().attr("href").context("no href in a tag")?;
        let user = &user[1..];
        let lgtm = e[4].inner_html();
        let lgtm = lgtm
            .parse()
            .with_context(|| format!("cannot parse lgtm field: {}", lgtm))?;

        info!("Found user: {}, lgtm: {}", user, lgtm);

        if ignore.skip(user) {
            warn!("Skip user {}", user);
            continue;
        }

        let (bytes, ext) = if ignore.skip_image(user) {
            warn!("Using default image for user {}", user);
            (include_bytes!("default.png").to_vec(), "png".into())
        } else {
            let body = fetch_page(&format!("https://qiita.com/{}", user)).await?;
            let doc = Html::parse_document(&body);
            let img = doc
                .select(&imgsel)
                .next()
                .expect("no img tag")
                .value()
                .attr("src")
                .expect("no href in img tag");
            fetch_image(img).await?
        };

        let path = opt.downloaded_dir.join(format!("{}.{}", user, ext));
        info!("Writing image to {}", path.display());
        tokio::fs::write(&path, bytes)
            .await
            .with_context(|| format!("cannot write to {}", path.display()))?;

        enemies.enemies.push(EnemyInfo {
            user: user.to_string(),
            lgtm,
            ext,
        });
    }

    tokio::fs::write(
        &opt.output_info,
        serde_json::to_vec(&enemies).context("cannot serialize info")?,
    )
    .await
    .with_context(|| format!("cannot write the info to {}", opt.output_info.display()))?;

    Ok(())
}

async fn process_images(opt: &Opt) -> Result<()> {
    let ignore = load_ignore_list(opt)?;

    tokio::fs::create_dir_all(&opt.processed_dir).await?;

    let mut images = vec![];

    let list: EnemyList = serde_json::from_reader(
        std::fs::File::open(&opt.output_info)
            .with_context(|| format!("cannot open info file: {}", opt.output_info.display()))?,
    )
    .with_context(|| format!("cannot parse info file: {}", opt.output_info.display()))?;

    for entry in list.enemies {
        let path = opt
            .downloaded_dir
            .join(&format!("{}.{}", entry.user, entry.ext));

        info!("Prcessing image: {}", path.display());
        let img = if ignore.skip_image(&entry.user) {
            warn!("Using default image for user {}", entry.user);
            image::load_from_memory(include_bytes!("default.png")).unwrap()
        } else {
            image::open(&path)
                .with_context(|| format!("cannot open image at {}", path.display()))?
        };
        let img = img.resize(opt.width, opt.height, image::imageops::FilterType::Nearest);

        let outpath = opt.processed_dir.join(&format!("{}.png", entry.user));
        img.save(&outpath)
            .with_context(|| format!("cannot save image to {}", outpath.display()))?;
        images.push(img);
    }

    use image::GenericImageView;

    let mut combined = image::ImageBuffer::new(images.len() as u32 * 32, 32);
    for (x, y, pixel) in combined.enumerate_pixels_mut() {
        let img = &images[x as usize / 32];
        if img.in_bounds(x % 32, y) {
            let img_pixel = img.get_pixel(x % 32, y);
            *pixel = img_pixel;
        }
    }

    info!("Merging images to {}", opt.output_texture.display());

    combined.save(&opt.output_texture).with_context(|| {
        format!(
            "cannot write the texture to {}",
            opt.output_texture.display()
        )
    })?;

    info!("Merged {} images", images.len());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();

    if !opt.process_only {
        download_images(&opt).await?;
    }

    process_images(&opt).await?;

    Ok(())
}
