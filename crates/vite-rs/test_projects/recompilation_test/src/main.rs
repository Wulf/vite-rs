#[derive(vite_rs::Embed)]
#[root = "./app"]
struct Assets;

fn main() {
    for asset in Assets::iter() {
        println!("{}", asset);
    }
}
