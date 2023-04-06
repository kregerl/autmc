use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub fn parse_options_txt(instance_dir: &Path) -> io::Result<HashMap<String, String>> {
    let options_txt_path = instance_dir.join("options.txt");
    let file = File::open(options_txt_path)?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .into_iter()
        .map(|line_res| {
            let line = line_res?;
            let splits: Vec<&str> = line.split(":").collect();
            Ok((splits[0].to_owned(), splits[1].to_owned()))
        })
        .collect()
}

fn copy_applicable_options(src_options: HashMap<String, String>, dst_keys: &[String]) -> HashMap<String, String> {
    src_options.into_iter().filter(|(key, _)| dst_keys.contains(key)).collect()
}

#[test]
fn test_parse_options_txt() {
    let instance_path = Path::new("C:\\Users\\kregerl\\AppData\\Roaming\\com.autm.launcher\\instances\\All the Mods 8\\");
    println!("Options: {:#?}", parse_options_txt(&instance_path));
}
