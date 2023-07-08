use std::{fmt, str::FromStr};

use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize)]
pub enum KonachanRatingFilter {
    Safe,
    Questionable,
    Explicit,
    QuestionableAndExplicit,
    QuestionableAndSafe,
}

impl fmt::Display for KonachanRatingFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KonachanRatingFilter::Safe => write!(f, "safe"),
            KonachanRatingFilter::Questionable => write!(f, "questionable"),
            KonachanRatingFilter::Explicit => write!(f, "explicit"),
            KonachanRatingFilter::QuestionableAndExplicit => write!(f, "questionableplus"),
            KonachanRatingFilter::QuestionableAndSafe => write!(f, "questionableless"),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum KonachanRatingFilterParseError {
    VariantNotFound,
}

impl std::fmt::Display for KonachanRatingFilterParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            KonachanRatingFilterParseError::VariantNotFound => write!(f, "Matching variant not found, must be one of: Safe|s, Questionable|q, Explicit|e, QuestionableAndExplicit|questionableplus|qe, QuestionableAndSafe|questionableless|qs"),
        }
    }
}

impl FromStr for KonachanRatingFilter {
    type Err = KonachanRatingFilterParseError;

    fn from_str(input: &str) -> Result<KonachanRatingFilter, Self::Err> {
        match input {
            "Safe" | "safe" | "s" => Ok(KonachanRatingFilter::Safe),
            "Questionable" | "questionable" | "q" => Ok(KonachanRatingFilter::Questionable),
            "Explicit" | "explicit" | "e" => Ok(KonachanRatingFilter::Explicit),
            "QuestionableAndExplicit" | "questionableplus" | "qe" => {
                Ok(KonachanRatingFilter::QuestionableAndExplicit)
            }
            "QuestionableAndSafe" | "questionableless" | "qs" => {
                Ok(KonachanRatingFilter::QuestionableAndSafe)
            }
            _ => Err(KonachanRatingFilterParseError::VariantNotFound),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize)]
pub enum Rating {
    Safe,
    Questionable,
    Explicit,
}

#[derive(Debug, thiserror::Error)]
pub enum RatingParseError {
    VariantNotFound,
}

impl std::fmt::Display for RatingParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            RatingParseError::VariantNotFound => write!(f, "Matching variant not found, must be one of: Safe|safe|s, Questionable|questionable|q, Explicit|explicit|e"),
        }
    }
}

impl FromStr for Rating {
    type Err = RatingParseError;

    fn from_str(input: &str) -> Result<Rating, Self::Err> {
        match input {
            "Safe" | "safe" | "s" => Ok(Rating::Safe),
            "Questionable" | "questionable" | "q" => Ok(Rating::Questionable),
            "Explicit" | "explicit" | "e" => Ok(Rating::Explicit),
            _ => Err(RatingParseError::VariantNotFound),
        }
    }
}

impl<'de> Deserialize<'de> for Rating {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Rating::Safe => write!(f, "Safe"),
            Rating::Questionable => write!(f, "Questionable"),
            Rating::Explicit => write!(f, "Explicit"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize)]
pub enum Category {
    Liked,
    Disliked,
    Borked,
}

#[derive(Debug, thiserror::Error)]
pub enum CategoryParseError {
    VariantNotFound,
}

impl std::fmt::Display for CategoryParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            CategoryParseError::VariantNotFound => write!(
                f,
                "Matching variant not found, must be one of: Liked, Disliked, Borked"
            ),
        }
    }
}

impl FromStr for Category {
    type Err = CategoryParseError;

    fn from_str(input: &str) -> Result<Category, Self::Err> {
        match input {
            "Liked" | "liked" | "l" => Ok(Category::Liked),
            "Disliked" | "disliked" | "d" => Ok(Category::Disliked),
            "Borked" | "borked" | "b" => Ok(Category::Borked),
            _ => Err(CategoryParseError::VariantNotFound),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Category::Liked => write!(f, "Liked"),
            Category::Disliked => write!(f, "Disliked"),
            Category::Borked => write!(f, "Borked"),
        }
    }
}

impl TryFrom<CategoryPrompt> for Category {
    type Error = CategoryParseError;

    fn try_from(value: CategoryPrompt) -> Result<Self, Self::Error> {
        match value {
            CategoryPrompt::Liked => Ok(Category::Liked),
            CategoryPrompt::Disliked => Ok(Category::Disliked),
            CategoryPrompt::Borked => Ok(Category::Borked),
            _ => Err(CategoryParseError::VariantNotFound),
        }
    }
}
impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, Serialize)]
pub enum CategoryPrompt {
    Liked,
    Disliked,
    NeedsCropping,
    DidNotNeedCropping,
    Borked,
}

impl fmt::Display for CategoryPrompt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CategoryPrompt::Liked => write!(f, "I like it "),
            CategoryPrompt::Disliked => write!(f, "I don't like it"),
            CategoryPrompt::NeedsCropping => {
                write!(f, "Do smart cropping, and go show me when it's done")
            }
            CategoryPrompt::DidNotNeedCropping => {
                write!(f, "Did not need cropping, go back to uncropped")
            }
            CategoryPrompt::Borked => write!(f, "It's borked"),
        }
    }
}
