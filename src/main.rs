mod yalt;

use structopt::StructOpt;
use yalt::Opt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let db_file = yalt::generate_db_filename(&opt);
    let db_path = std::path::Path::new("databases");
    std::fs::create_dir_all(db_path)?;
    let db_full_path = db_path.join(db_file);

    yalt::run_yalt(opt, &db_full_path).await
}
