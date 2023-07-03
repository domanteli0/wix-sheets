mod data;

fn main() {
    let data_str = include_str!("../example.json");
    let data_raw: data::RawData = serde_json::from_str(data_str).unwrap();
    
    println!("{:?}", data_raw);
}
