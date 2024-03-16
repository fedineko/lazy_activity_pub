use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

/// This structure represents image data structure
/// that is not a complete ActivityPub object, note the absence of
/// context and entity type.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Image {
    /// Link to actual image data.
    pub url: Option<url::Url>,

    /// Description of image or just name of it.
    #[serde(alias = "name")]
    pub summary: Option<String>,

    #[serde(alias = "mediaType")]
    pub media_type: Option<String>,

    /// Does this image have explicit or otherwise sensitive content?
    /// It is not very reliable property as nothing (but moderators)
    /// stops folks from ignoring this property.
    pub sensitive: Option<bool>,

    /// Width of image in pixels.
    pub width: Option<u32>,

    /// Height of image in pixels.
    pub height: Option<u32>,
}

impl Image {
    /// This method constructs new instance of Image object with [url]
    /// as location to get image data. All other properties are not set.
    pub fn from_url(url: url::Url) -> Self {
        Self {
            url: Some(url),
            summary: None,
            media_type: None,
            sensitive: None,
            width: None,
            height: None,
        }
    }
}

/// This enumeration helps tp deals with multiple ways to reference
/// Image by different services.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ImageReference {
    /// Property is direct link to image data.
    Url(url::Url),

    /// Property is [Image] object describing image.
    Single(Image),

    /// Property is set of [Image] objects.
    List(Vec<Image>),
}

impl ImageReference {
    /// Consumes self and returns vector of [Image] objects.
    pub fn to_vec(self) -> Vec<Image> {
        match self {
            Self::Single(icon) => vec![icon],
            Self::List(icons) => icons,
            Self::Url(url)=> vec![Image::from_url(url)],
        }
    }

    /// This helper method tries to figure out the largest image
    /// declared in this [ImageReference]. Returns either the largest
    /// image or jany image if it is not possible to infer the largest,
    /// or `None` if there is no any image definitions in this reference.
    pub fn get_largest_image(self) -> Option<Image> {
        let mut images = self.to_vec();

        images.sort_by(|b, a| {
            if a.width.is_some() && b.width.is_some() {
                let a_width = a.width.as_ref().unwrap();
                let b_width = b.width.as_ref().unwrap();

                return a_width.cmp(b_width);
            }

            if a.height.is_some() && b.height.is_some() {
                let a_height = a.height.as_ref().unwrap();
                let b_height = b.height.as_ref().unwrap();

                return a_height.cmp(b_height);
            }

            Ordering::Equal
        });

        images.into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use crate::image::ImageReference;

    #[test]
    fn test_image_ordering_is_correct() {
        let serialized = r#"[
    {
      "type": "Image",
      "url": "https://peertube.stream/lazy-static/thumbnails/xxx.jpg",
      "mediaType": "image/jpeg",
      "width": 280,
      "height": 157
    },
    {
      "type": "Image",
      "url": "https://peertube.stream/lazy-static/previews/yyy.jpg",
      "mediaType": "image/jpeg",
      "width": 850,
      "height": 480
    }
  ]"#;
        let image_reference: ImageReference = serde_json::from_str(serialized).unwrap();
        let image = image_reference.get_largest_image().unwrap();
        assert_eq!(850, image.width.unwrap());
    }
}
