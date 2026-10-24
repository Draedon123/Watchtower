use std::{collections::HashMap, sync::Arc};

use http::{method::HTTPMethod, response::Response};

use crate::server::ServerRequest;

pub enum URLSection {
  Literal(String),
  Slug(String),
}

pub struct Route {
  pub method: HTTPMethod,
  pub path: Vec<URLSection>,
  pub callback: Arc<dyn Fn(&mut ServerRequest, &mut Response) + Send + Sync>,
}

impl Route {
  pub fn path_from_url(url: &str) -> Vec<URLSection> {
    let split_url: Vec<_> = url.split("/").collect();
    let mut path: Vec<URLSection> = Vec::with_capacity(split_url.len());

    split_url.iter().for_each(|section| {
      if section.starts_with("[") && section.ends_with("]") {
        let slug_name = (&section[1..(section.len() - 1)]).to_string();
        path.push(URLSection::Slug(slug_name));
      } else {
        path.push(URLSection::Literal(section.to_string()));
      };
    });

    path
  }

  pub fn matches(&self, url: &str) -> Option<HashMap<String, String>> {
    let split_url: Vec<_> = url.split("/").collect();

    if split_url.len() != self.path.len() {
      None
    } else {
      let mut slugs: HashMap<String, String> = HashMap::new();

      for i in 0..split_url.len() {
        match &self.path[i] {
          URLSection::Literal(section) => {
            if section != split_url[i] {
              return None;
            }
          }
          URLSection::Slug(slug) => {
            slugs.insert(slug.clone(), split_url[i].to_string());
          }
        }
      }

      Some(slugs)
    }
  }
}
