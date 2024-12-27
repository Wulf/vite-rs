#[derive(vite_rs::Embed)]
#[root = "."]
#[output = "./custom-output-dir/dist"]
struct Assets;

fn main() {
    let _guard = Assets::start_dev_server(true);

    std::thread::sleep(std::time::Duration::from_secs(12));
}
