use crate::model::{MediaMetadata, MediaType, SearchResult};
use serde::Deserialize;

const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";

/// TMDB API client
pub struct TmdbClient {
    api_key: String,
    client: reqwest::Client,
}

impl TmdbClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Verify API key is valid by making a test request
    pub async fn verify_api_key(&self) -> bool {
        let url = format!(
            "{}/configuration?api_key={}",
            TMDB_BASE_URL,
            self.api_key
        );

        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Search for movies
    pub async fn search_movies(&self, query: &str, year: Option<u32>) -> Result<Vec<SearchResult>, String> {
        let mut url = format!(
            "{}/search/movie?api_key={}&query={}&include_adult=false",
            TMDB_BASE_URL,
            self.api_key,
            urlencoding::encode(query)
        );

        if let Some(y) = year {
            url.push_str(&format!("&year={}", y));
        }

        let response: TmdbSearchResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(response
            .results
            .into_iter()
            .map(|r| SearchResult {
                tmdb_id: r.id,
                title: r.title.unwrap_or_else(|| r.name.unwrap_or_default()),
                year: r.release_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok()),
                media_type: MediaType::Movie,
                overview: r.overview,
                poster_path: r.poster_path,
                vote_average: r.vote_average,
            })
            .collect())
    }

    /// Search for TV shows
    pub async fn search_tv(&self, query: &str, year: Option<u32>) -> Result<Vec<SearchResult>, String> {
        let mut url = format!(
            "{}/search/tv?api_key={}&query={}&include_adult=false",
            TMDB_BASE_URL,
            self.api_key,
            urlencoding::encode(query)
        );

        if let Some(y) = year {
            url.push_str(&format!("&first_air_date_year={}", y));
        }

        let response: TmdbSearchResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(response
            .results
            .into_iter()
            .map(|r| SearchResult {
                tmdb_id: r.id,
                title: r.name.unwrap_or_else(|| r.title.unwrap_or_default()),
                year: r.first_air_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok()),
                media_type: MediaType::TvShow,
                overview: r.overview,
                poster_path: r.poster_path,
                vote_average: r.vote_average,
            })
            .collect())
    }

    /// Multi-search (movies and TV shows)
    pub async fn search_multi(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!(
            "{}/search/multi?api_key={}&query={}&include_adult=false",
            TMDB_BASE_URL,
            self.api_key,
            urlencoding::encode(query)
        );

        let response: TmdbSearchResponse = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(response
            .results
            .into_iter()
            .filter(|r| r.media_type.as_deref() == Some("movie") || r.media_type.as_deref() == Some("tv"))
            .map(|r| {
                let is_movie = r.media_type.as_deref() == Some("movie");
                SearchResult {
                    tmdb_id: r.id,
                    title: if is_movie {
                        r.title.unwrap_or_else(|| r.name.unwrap_or_default())
                    } else {
                        r.name.unwrap_or_else(|| r.title.unwrap_or_default())
                    },
                    year: if is_movie {
                        r.release_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok())
                    } else {
                        r.first_air_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok())
                    },
                    media_type: if is_movie { MediaType::Movie } else { MediaType::TvShow },
                    overview: r.overview,
                    poster_path: r.poster_path,
                    vote_average: r.vote_average,
                }
            })
            .collect())
    }

    /// Get movie details
    pub async fn get_movie_details(&self, movie_id: u64) -> Result<MediaMetadata, String> {
        let url = format!(
            "{}/movie/{}?api_key={}",
            TMDB_BASE_URL,
            movie_id,
            self.api_key
        );

        let movie: TmdbMovieDetails = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(MediaMetadata {
            tmdb_id: movie.id,
            title: movie.title,
            original_title: movie.original_title,
            year: movie.release_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok()),
            overview: movie.overview,
            poster_path: movie.poster_path,
            backdrop_path: movie.backdrop_path,
            vote_average: movie.vote_average,
            genres: movie.genres.into_iter().map(|g| g.name).collect(),
            ..Default::default()
        })
    }

    /// Get TV show details
    pub async fn get_tv_details(&self, tv_id: u64) -> Result<MediaMetadata, String> {
        let url = format!(
            "{}/tv/{}?api_key={}",
            TMDB_BASE_URL,
            tv_id,
            self.api_key
        );

        let tv: TmdbTvDetails = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(MediaMetadata {
            tmdb_id: tv.id,
            title: tv.name.clone(),
            original_title: tv.original_name,
            year: tv.first_air_date.as_ref().and_then(|d| d.split('-').next()?.parse().ok()),
            overview: tv.overview,
            poster_path: tv.poster_path,
            backdrop_path: tv.backdrop_path,
            vote_average: tv.vote_average,
            genres: tv.genres.into_iter().map(|g| g.name).collect(),
            show_name: Some(tv.name),
            ..Default::default()
        })
    }

    /// Get TV episode details
    pub async fn get_episode_details(
        &self,
        tv_id: u64,
        season: u32,
        episode: u32,
    ) -> Result<MediaMetadata, String> {
        // First get the show details
        let show = self.get_tv_details(tv_id).await?;

        // Then get the episode details
        let url = format!(
            "{}/tv/{}/season/{}/episode/{}?api_key={}",
            TMDB_BASE_URL,
            tv_id,
            season,
            episode,
            self.api_key
        );

        let ep: TmdbEpisodeDetails = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(MediaMetadata {
            tmdb_id: tv_id,
            title: show.title.clone(),
            original_title: show.original_title,
            year: show.year,
            overview: ep.overview,
            poster_path: ep.still_path.or(show.poster_path),
            backdrop_path: show.backdrop_path,
            vote_average: ep.vote_average.or(show.vote_average),
            genres: show.genres,
            season_number: Some(season),
            episode_number: Some(episode),
            episode_title: Some(ep.name),
            air_date: ep.air_date,
            show_name: show.show_name,
        })
    }
}

// TMDB API response types

#[derive(Deserialize)]
struct TmdbSearchResponse {
    results: Vec<TmdbSearchResult>,
}

#[derive(Deserialize)]
struct TmdbSearchResult {
    id: u64,
    title: Option<String>,
    name: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    release_date: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f32>,
    media_type: Option<String>,
}

#[derive(Deserialize)]
struct TmdbMovieDetails {
    id: u64,
    title: String,
    original_title: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    release_date: Option<String>,
    vote_average: Option<f32>,
    genres: Vec<TmdbGenre>,
}

#[derive(Deserialize)]
struct TmdbTvDetails {
    id: u64,
    name: String,
    original_name: Option<String>,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    first_air_date: Option<String>,
    vote_average: Option<f32>,
    genres: Vec<TmdbGenre>,
}

#[derive(Deserialize)]
struct TmdbEpisodeDetails {
    name: String,
    overview: Option<String>,
    still_path: Option<String>,
    air_date: Option<String>,
    vote_average: Option<f32>,
}

#[derive(Deserialize)]
struct TmdbGenre {
    name: String,
}

/// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push_str("%20"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}

/// Verify if an API key is valid
pub async fn verify_api_key(api_key: &str) -> bool {
    if api_key.is_empty() {
        return false;
    }
    let client = TmdbClient::new(api_key.to_string());
    client.verify_api_key().await
}

/// Search for media (auto-detect type)
pub async fn search_media(
    api_key: &str,
    query: &str,
    media_type: MediaType,
    year: Option<u32>,
) -> Result<Vec<SearchResult>, String> {
    if api_key.is_empty() {
        return Err("TMDB API key not set".to_string());
    }

    let client = TmdbClient::new(api_key.to_string());

    match media_type {
        MediaType::Movie => client.search_movies(query, year).await,
        MediaType::TvShow => client.search_tv(query, year).await,
        MediaType::Unknown => client.search_multi(query).await,
    }
}

/// Fetch full metadata for a search result
pub async fn fetch_metadata(
    api_key: &str,
    result: &SearchResult,
    season: Option<u32>,
    episode: Option<u32>,
) -> Result<MediaMetadata, String> {
    if api_key.is_empty() {
        return Err("TMDB API key not set".to_string());
    }

    let client = TmdbClient::new(api_key.to_string());

    match result.media_type {
        MediaType::Movie => client.get_movie_details(result.tmdb_id).await,
        MediaType::TvShow => {
            if let (Some(s), Some(e)) = (season, episode) {
                // Try to get episode details, fall back to TV show details with season/episode info
                match client.get_episode_details(result.tmdb_id, s, e).await {
                    Ok(metadata) => Ok(metadata),
                    Err(_) => {
                        // Episode not found on TMDB, get show details and add season/episode from parsed info
                        let mut metadata = client.get_tv_details(result.tmdb_id).await?;
                        metadata.season_number = Some(s);
                        metadata.episode_number = Some(e);
                        Ok(metadata)
                    }
                }
            } else {
                client.get_tv_details(result.tmdb_id).await
            }
        }
        MediaType::Unknown => {
            // For unknown type, try multi-search approach - default to movie
            client.get_movie_details(result.tmdb_id).await
        }
    }
}

/// File info for batch matching
#[derive(Clone)]
pub struct BatchFileInfo {
    pub index: usize,
    pub title: String,
    pub year: Option<u32>,
    pub season: Option<u32>,
    pub episode: Option<u32>,
    pub media_type: MediaType,
}

/// Optimized batch matching - groups by title, fetches show once, episodes in parallel
pub async fn batch_match_files(
    api_key: &str,
    files: Vec<BatchFileInfo>,
) -> Vec<(usize, Result<MediaMetadata, String>)> {
    use std::collections::HashMap;
    use futures::future::join_all;
    
    if api_key.is_empty() {
        return files.iter().map(|f| (f.index, Err("TMDB API key not set".to_string()))).collect();
    }

    let client = TmdbClient::new(api_key.to_string());
    let mut results: Vec<(usize, Result<MediaMetadata, String>)> = Vec::new();
    
    // Separate movies and TV shows
    let mut movies: Vec<BatchFileInfo> = Vec::new();
    let mut tv_shows: HashMap<String, Vec<BatchFileInfo>> = HashMap::new();
    
    for file in files {
        if file.title.is_empty() {
            results.push((file.index, Err("No title parsed".to_string())));
            continue;
        }
        
        match file.media_type {
            MediaType::Movie | MediaType::Unknown => movies.push(file),
            MediaType::TvShow => {
                let key = file.title.to_lowercase();
                tv_shows.entry(key).or_default().push(file);
            }
        }
    }
    
    // Process movies (each needs individual search)
    for movie in movies {
        match client.search_movies(&movie.title, movie.year).await {
            Ok(search_results) if !search_results.is_empty() => {
                match client.get_movie_details(search_results[0].tmdb_id).await {
                    Ok(metadata) => results.push((movie.index, Ok(metadata))),
                    Err(e) => results.push((movie.index, Err(e))),
                }
            }
            Ok(_) => results.push((movie.index, Err("No results found".to_string()))),
            Err(e) => results.push((movie.index, Err(e))),
        }
        // Small delay for movies
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Process TV shows - grouped by title
    for (_, episodes) in tv_shows {
        if episodes.is_empty() {
            continue;
        }
        
        let first = &episodes[0];
        
        // Search for the show ONCE
        let search_result = match client.search_tv(&first.title, first.year).await {
            Ok(r) if !r.is_empty() => r[0].clone(),
            Ok(_) => {
                for ep in &episodes {
                    results.push((ep.index, Err("No results found".to_string())));
                }
                continue;
            }
            Err(e) => {
                for ep in &episodes {
                    results.push((ep.index, Err(e.clone())));
                }
                continue;
            }
        };
        
        // Get show details ONCE
        let show_details = match client.get_tv_details(search_result.tmdb_id).await {
            Ok(d) => d,
            Err(e) => {
                for ep in &episodes {
                    results.push((ep.index, Err(e.clone())));
                }
                continue;
            }
        };
        
        // Fetch all episodes concurrently (in batches to avoid rate limits)
        let batch_size = 5; // 5 concurrent requests
        for chunk in episodes.chunks(batch_size) {
            let futures: Vec<_> = chunk.iter().map(|ep| {
                let client = TmdbClient::new(api_key.to_string());
                let tmdb_id = search_result.tmdb_id;
                let show = show_details.clone();
                let season = ep.season;
                let episode_num = ep.episode;
                let index = ep.index;
                
                async move {
                    if let (Some(s), Some(e)) = (season, episode_num) {
                        // Fetch episode directly (without re-fetching show details)
                        let url = format!(
                            "{}/tv/{}/season/{}/episode/{}?api_key={}",
                            TMDB_BASE_URL,
                            tmdb_id,
                            s,
                            e,
                            client.api_key
                        );
                        
                        match client.client.get(&url).send().await {
                            Ok(response) => {
                                match response.json::<TmdbEpisodeDetails>().await {
                                    Ok(ep_details) => {
                                        let metadata = MediaMetadata {
                                            tmdb_id,
                                            title: show.title.clone(),
                                            original_title: show.original_title.clone(),
                                            year: show.year,
                                            overview: ep_details.overview,
                                            poster_path: ep_details.still_path.or(show.poster_path.clone()),
                                            backdrop_path: show.backdrop_path.clone(),
                                            vote_average: ep_details.vote_average.or(show.vote_average),
                                            genres: show.genres.clone(),
                                            season_number: Some(s),
                                            episode_number: Some(e),
                                            episode_title: Some(ep_details.name),
                                            air_date: ep_details.air_date,
                                            show_name: show.show_name.clone(),
                                        };
                                        (index, Ok(metadata))
                                    }
                                    Err(_) => {
                                        // Episode not found, use show details with parsed info
                                        let mut metadata = show.clone();
                                        metadata.season_number = Some(s);
                                        metadata.episode_number = Some(e);
                                        (index, Ok(metadata))
                                    }
                                }
                            }
                            Err(e) => (index, Err(format!("Network error: {}", e))),
                        }
                    } else {
                        (index, Ok(show.clone()))
                    }
                }
            }).collect();
            
            let batch_results = join_all(futures).await;
            results.extend(batch_results);
            
            // Small delay between batches
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    results
}

