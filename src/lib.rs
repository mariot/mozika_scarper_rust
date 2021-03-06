extern crate reqwest;
extern crate select;

use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate, Text};

pub struct Artist {
    pub id: i32,
    pub name: String,
    pub url: String,
    pub songs_id: i32,
    pub number_of_songs: i32,
}

pub struct Song {
    pub id: i32,
    pub title: String,
    pub url: String,
    pub lyrics: String,
}

fn get_artists_page_infos(url: &str) -> Vec<Artist> {
    let resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());

    println!("Current artists page: {}", url);

    let document = Document::from_read(resp).unwrap();

    let mut artists: Vec<Artist> = Vec::new();

    for node in document.find(
        Class("list")
            .descendant(Name("tbody"))
            .descendant(Name("tr")),
    ) {
        let id = node
            .find(Name("td"))
            .next()
            .unwrap()
            .text()
            .parse()
            .expect("ID number invalid");
        let name = node.find(Name("td").descendant(Name("a"))).next().unwrap();
        let url_txt = name.attr("href").unwrap();
        let url = url_txt.trim_left_matches('/');

        let songs_url_node = node
            .find(Attr("align", "right").descendant(Name("a")))
            .next()
            .unwrap();
        let mut songs_url = songs_url_node
            .attr("href")
            .unwrap()
            .trim_left_matches('/')
            .to_string();
        songs_url.pop();
        songs_url.pop();
        let songs_id: i32 = songs_url[51..].parse().expect("Songs ID number invalid");
        let number_of_songs: i32 = songs_url_node.text().parse().expect("Song number invalid");

        artists.push(Artist {
            id,
            name: name.text(),
            url: url.to_string()[38..].to_string(),
            songs_id,
            number_of_songs,
        });
    }

    artists
}

fn get_songs_page_infos(url: &str) -> Vec<Song> {
    let resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());

    let document = Document::from_read(resp).unwrap();

    let mut songs: Vec<Song> = Vec::new();

    for node in document.find(
        Class("list")
            .descendant(Name("tbody"))
            .descendant(Name("tr")),
    ) {
        let id: i32 = node
            .find(Name("td"))
            .next()
            .unwrap()
            .text()
            .parse()
            .expect("ID number invalid");
        let title = node.find(Name("td").descendant(Name("a"))).next().unwrap();
        let url = title.attr("href").unwrap().trim_left_matches('/');
        let lyrics = get_song_lyrics(&url);
        if lyrics.is_empty() {
            println!("[!!!] Could not parse: {}", url);
            continue;
        }

        songs.push(Song {
            id,
            title: title.text(),
            url: url.to_string()[35..].to_string(),
            lyrics,
        });
    }

    songs
}

fn get_song_lyrics(url: &str) -> String {
    let resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());

    let mut lyrics = String::new();

    match Document::from_read(resp) {
        Ok(document) => {
            let lyrics_node = document.find(Class("l-2-3")).next();
            if lyrics_node != None {
                let lyrics_html = lyrics_node
                    .unwrap()
                    .children()
                    .filter(|child| child.is(Text))
                    .map(|node| node.text());

                for lyrics_line in lyrics_html {
                    let line_to_add = lyrics_line.as_str().trim().to_string() + "\n";
					if line_to_add != "\n\n" {
                    	lyrics.push_str(&line_to_add.as_str());
					}
                }
            }
        }
        Err(_) => {
            eprintln!("{}", url);
        }
    }

    lyrics
}

pub fn scrap_artists(page: i32) -> Vec<Artist> {
    let url = format!(
        "{}/{}",
        "http://tononkira.serasera.org/tononkira/mpihira/results",
        page.to_string()
    );

    get_artists_page_infos(&url)
}

pub fn scrap_songs(id: i32) -> Vec<Song> {
    let mut songs: Vec<Song> = Vec::new();
    let mut page = 0;
    loop {
        let url = format!(
            "{}/{}/{}",
            "http://tononkira.serasera.org/tononkira/hira/index",
            id.to_string(),
            page.to_string()
        );
        let songs_on_this_page = get_songs_page_infos(&url);
        if songs_on_this_page.is_empty() {
            break;
        }
        songs.extend(songs_on_this_page);
        page += 20;
    }

    songs
}
