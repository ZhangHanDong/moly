use makepad_widgets::{Action, Cx};
use moly_kit::utils::asynchronous::spawn;
use moly_protocol::data::*;

use super::moly_client::MolyClient;

#[derive(Clone, Copy, Debug, Default)]
pub enum SortCriteria {
    #[default]
    MostDownloads,
    LeastDownloads,
    MostLikes,
    LeastLikes,
}

#[derive(Debug)]
pub enum SearchAction {
    Results(Vec<Model>),
    Error,
}

#[derive(Clone)]
pub enum SearchCommand {
    Search(String),
    LoadFeaturedModels,
}

#[derive(Default, Clone)]
pub enum SearchState {
    #[default]
    Idle,
    Pending(SearchCommand, Option<SearchCommand>),
    Errored,
}
pub struct Search {
    pub moly_client: MolyClient,
    pub models: Vec<Model>,
    pub sorted_by: SortCriteria,
    pub keyword: Option<String>,
    pub state: SearchState,
}

impl Search {
    pub fn new(moly_client: MolyClient) -> Self {
        let search = Self {
            moly_client,
            models: Vec::new(),
            sorted_by: SortCriteria::MostDownloads,
            keyword: None,
            state: SearchState::Idle,
        };
        search
    }

    pub fn load_featured_models(&mut self) {
        match self.state {
            SearchState::Pending(_, ref mut next_command) => {
                *next_command = Some(SearchCommand::LoadFeaturedModels);
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending(SearchCommand::LoadFeaturedModels, None);
                self.keyword = None;
            }
        }

        let moly_client = self.moly_client.clone();
        spawn(async move {
            match moly_client.get_featured_models().await {
                Ok(models) => {
                    Cx::post_action(SearchAction::Results(models));
                }
                Err(_err) => {
                    Cx::post_action(SearchAction::Error);
                }
            }
        });
    }

    pub fn load_search_results(&mut self, query: String) {
        self.run_or_enqueue(query);
    }

    fn run_or_enqueue(&mut self, keyword: String) {
        match self.state {
            SearchState::Pending(_, ref mut next_command) => {
                *next_command = Some(SearchCommand::Search(keyword));
                return;
            }
            SearchState::Idle | SearchState::Errored => {
                self.state = SearchState::Pending(SearchCommand::Search(keyword.clone()), None);
            }
        }

        let moly_client = self.moly_client.clone();
        spawn(async move {
            match moly_client.search_models(keyword.clone()).await {
                Ok(models) => {
                    Cx::post_action(SearchAction::Results(models));
                }
                Err(err) => {
                    eprintln!("Error fetching models: {:?}", err);
                    Cx::post_action(SearchAction::Error);
                }
            }
        });
    }

    pub fn sort_models(&mut self, criteria: SortCriteria) {
        match criteria {
            SortCriteria::MostDownloads => {
                self.models
                    .sort_by(|a, b| b.download_count.cmp(&a.download_count));
            }
            SortCriteria::LeastDownloads => {
                self.models
                    .sort_by(|a, b| a.download_count.cmp(&b.download_count));
            }
            SortCriteria::MostLikes => {
                self.models.sort_by(|a, b| b.like_count.cmp(&a.like_count));
            }
            SortCriteria::LeastLikes => {
                self.models.sort_by(|a, b| a.like_count.cmp(&b.like_count));
            }
        }
        self.sorted_by = criteria;
    }

    pub fn set_models(&mut self, models: Vec<Model>) {
        self.models = models;
        self.sort_models(self.sorted_by);
    }

    pub fn handle_action(&mut self, action: &Action) {
        if let Some(msg) = action.downcast_ref::<SearchAction>() {
            match msg {
                SearchAction::Results(models) => {
                    let previous_state = self.state.to_owned();
                    self.state = SearchState::Idle;

                    if let SearchState::Pending(current_command, next_command) = previous_state {
                        if let SearchCommand::Search(keyword) = current_command {
                            self.keyword = Some(keyword.clone());
                        }

                        match next_command {
                            Some(SearchCommand::Search(next_keyword)) => {
                                self.run_or_enqueue(next_keyword.clone());
                            }
                            Some(SearchCommand::LoadFeaturedModels) => {
                                self.load_featured_models();
                            }
                            None => {}
                        }
                        self.set_models(models.clone());
                    } else {
                        self.set_models(vec![]);
                        eprintln!("Client was not expecting to receive results");
                    }
                }
                SearchAction::Error => {
                    self.state = SearchState::Errored;
                    self.set_models(vec![]);
                }
            }
        }
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.state, SearchState::Pending(_, _))
    }

    pub fn was_error(&self) -> bool {
        matches!(self.state, SearchState::Errored)
    }

    pub fn update_downloaded_file_in_search_results(&mut self, file_id: &FileID, downloaded: bool) {
        let model = self
            .models
            .iter_mut()
            .find(|m| m.files.iter().any(|f| f.id == *file_id));
        if let Some(model) = model {
            let file = model.files.iter_mut().find(|f| f.id == *file_id).unwrap();
            file.downloaded = downloaded;
        }
    }

    pub fn get_model_and_file_from_search_results(&self, file_id: &str) -> Option<(Model, File)> {
        self.models.iter().find_map(|m| {
            m.files
                .iter()
                .find(|f| f.id == file_id)
                .map(|f| (m.clone(), f.clone()))
        })
    }
}
