use std::io::{stdin, BufRead, BufReader};
use clap::{App, Arg, ArgGroup};
use std::fs::File;

fn main() {
    // オプション定義
    let app = App::new("ncut")
                .version("0.1.0")
                .about("cut by field name")
                .author("Mugi_GrainP")
                .arg(Arg::with_name("file")
                     .help("ファイル名")
                 )
                .arg(Arg::with_name("delim")
                     .help("区切り記号 (デフォルトは空白)")
                     .short("d")
                     .long("delimiter")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("field")
                     .help("フィールド識別子")
                     .short("f")
                     .long("field")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("field_by_title")
                     .help("フィールド識別子 (タイトル行を利用)")
                     .short("t")
                     .long("field-by-title")
                     .takes_value(true)
                 )
                .group(ArgGroup::with_name("field_specification")
                       .args(&["field", "field_by_title"])
                       .required(true)
                 );


    // オプション解析
    let matches = app.get_matches();

    let delimiter = match matches.value_of("delim") {
        None => " ",
        Some(o) => o
    };

    let viewfield_str: (i32, &str) = match matches.value_of("field") {
        None => (1, matches.value_of("field_by_title").unwrap()),
        Some(o) => (0, o),
    };

    if let Some(path) = matches.value_of("file") {
        let f = File::open(path).unwrap();
        let reader = BufReader::new(f);
        read_and_output(reader, delimiter, viewfield_str);
    } else {
        let stdin = stdin();
        let reader = stdin.lock();
        read_and_output(reader, delimiter, viewfield_str);
    }

}

// ファイル読み込み表示
fn read_and_output<R: BufRead>(mut reader: R, delimiter: &str, viewfield_str: (i32, &str)) {

    let mut first_line = String::new();
    reader.read_line(&mut first_line).expect("Can't read");
    first_line = first_line.trim_end().to_string();

    let field_count = first_line.split(delimiter).collect::<Vec<_>>().len();
    let viewfield = match viewfield_str.0 {
        0 => set_viewfield(field_count, viewfield_str.1),
        1 => set_viewfield(field_count,
                           &make_viewfield_str(first_line.split(delimiter).collect(), viewfield_str.1)),
        _ => panic!(),
    };

    split_and_print(&first_line, delimiter, &viewfield);

    for line in reader.lines() {
        let row = line.unwrap();
        split_and_print(&row, delimiter, &viewfield);
    }
}

fn split_and_print(row: &str, delimiter: &str, viewfield: &Vec<bool>) {
    let mut output: Vec<&str> = row.split(delimiter).collect();
    let mut i = 0;

    // viewfieldがtrueの値のみを残す
    output.retain(|_| (viewfield[i], i += 1).0);
    let output_line = output.join(delimiter);

    println!("{}", output_line);
}

fn set_viewfield(field_count: usize, field_str: &str) -> Vec<bool> {
    let token: Vec<&str> = field_str.split(',').collect();
    let mut viewfield: Vec<bool> = vec![false; field_count];

    for t in token {
        match t {
            x if x.find('-').is_some()  => {
                let from_to: Vec<&str> = x.split('-').collect();

                let fstart = usize::from_str_radix(from_to[0], 10).unwrap() ;
                let fend = usize::from_str_radix(from_to[1], 10).unwrap_or(field_count);

                let actual_start = if fstart <= field_count {
                    fstart
                } else {
                    field_count
                };

                let actual_end = if fend <= field_count {
                    fend
                } else {
                    field_count
                };

                for i in (actual_start - 1)..=(actual_end - 1) {
                    viewfield[i] = true;
                }
            },
            _ => {
                let n = usize::from_str_radix(t, 10).unwrap();
                if n <= field_count {
                    viewfield[n - 1] = true;
                }
            },
        }
    }

    viewfield
}

fn make_viewfield_str(header_list: Vec<&str>, fields: &str) -> String {
    let field_list: Vec<&str> = fields.split(',').collect();
    let mut viewfield: Vec<String> = vec![];

    for f in field_list {
        let mut header_list_iter = header_list.iter();
        match header_list_iter.position(|&x| x == f) {
            Some(idx) => viewfield.push((idx + 1).to_string()),
            None => (),
        }
    }

    viewfield.join(",")
}

