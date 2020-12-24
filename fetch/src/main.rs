use scraper::{Html, Selector};
use serde::Serialize;
use structopt::StructOpt;

#[derive(Serialize)]
struct EnemyInfo {
    user: String,
    lgtm: u32,
}

#[derive(Serialize)]
struct EnemyList {
    enemies: Vec<EnemyInfo>,
}

#[derive(StructOpt)]
struct Opt {
    #[structopt(short = "p", long = "process-only")]
    process_only: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let body = reqwest::get("https://qiita.com/torifukukaiou/items/c8361231cdc56e493245")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let doc = Html::parse_document(&body);
    let trsel = Selector::parse("tr").unwrap();
    let tdsel = Selector::parse("td").unwrap();
    let asel = Selector::parse("a").unwrap();
    let imgsel = Selector::parse("img").unwrap();

    let mut enemies = EnemyList { enemies: vec![] };
    let mut images = vec![];

    std::fs::create_dir_all("downloads").unwrap();
    std::fs::create_dir_all("processed").unwrap();

    for e in doc.select(&trsel) {
        println!("{}", e.inner_html());
        let e: Vec<_> = e.select(&tdsel).collect();
        if e.len() != 5 {
            continue;
        }

        let a = e[1].select(&asel).nth(1).expect("no 2nd a tag");
        let user = a.value().attr("href").expect("no href in a tag");
        let user = &user[1..];
        let lgtm = e[4].inner_html().parse().unwrap();
        println!("user: {}, lgtm: {}", user, lgtm);
        enemies.enemies.push(EnemyInfo {
            user: user.to_string(),
            lgtm,
        });

        let body = reqwest::get(&format!("https://qiita.com/{}", user))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let doc = Html::parse_document(&body);
        let img = doc
            .select(&imgsel)
            .next()
            .expect("no img tag")
            .value()
            .attr("src")
            .expect("no href in img tag");

        println!("img: {}", img);
        let resp = reqwest::get(img).await.unwrap();
        let ctype: &str = resp
            .headers()
            .get("content-type")
            .expect("cannot determine content type")
            .to_str()
            .expect("header value is not a string");

        let ext = match ctype {
            "image/gif" => "gif",
            "image/png" => "png",
            "image/jpg" | "image/jpeg" => "jpg",
            e => panic!("unsupported content type: {}", e),
        };

        let filename = format!("downloads/{}.{}", user, ext);
        let content = resp.bytes().await.unwrap();
        std::fs::write(&filename, content).unwrap();

        println!("processing");
        let img = image::open(&filename).unwrap();
        let img = img.resize(32, 32, image::imageops::FilterType::Nearest);
        println!("{:?}", img.bounds());
        img.save(&format!("processed/{}.png", user)).unwrap();
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
    combined
        .save("../static/textures/enemies_sheet.png")
        .unwrap();

    std::fs::write("../src/enemies.json", serde_json::to_vec(&enemies).unwrap()).unwrap();
}
