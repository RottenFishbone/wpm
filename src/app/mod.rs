pub mod view;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::prelude::SliceRandom;
use std::{collections::VecDeque, fs::File, io::{BufReader, BufRead}, iter::FromIterator, sync::mpsc::{self, Receiver, Sender}, time::{Duration, SystemTime}};

pub enum UIEvent {
    Input(KeyEvent),
    Tick,
}

/// Controller instance provides an interface layer for the game data
/// data can be read via the controller model but can only be modified
/// by the controller itself. 
/// The exit_tx member provides a shutdown channel for the whole program, that
/// is, if the user sends some form of kill signal, it will be delivered via
/// exit_rx.
pub struct Controller {
    pub model: Model,
    exit_tx: Sender<()>,
}

impl Controller {
    /// Create a new controller-model. The model is created as a member
    /// of the controller. The exit_rx channel will be transmitted to
    /// on close events sent by the controller.
    pub fn new() -> (Self, Receiver<()>) {
        let (exit_tx, exit_rx) = mpsc::channel();
        let controller = Self {
            model: Model::default(),
            exit_tx,
        };

        ( controller, exit_rx )
    }

    pub fn update(&mut self) {
        let elapsed = self.model.start.elapsed()
                                      .expect("Failed to get system time.");

        
        // Test if the timer has expired during an active round
        if self.model.round_state == RoundState::Active && 
            elapsed.as_millis() >= 30_000 {
            
            self.end_round();
        }
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Esc => self.exit_tx.send(()).expect("Failed to send exit signal."),
            KeyCode::Backspace => { self.model.word_typing.pop(); },
            KeyCode::Enter => {
                if self.model.round_state == RoundState::Completed {
                    self.model.word_queue = 
                        Model::new_word_list(&self.model.dict, 200);

                    self.model.chars_correct = 0;
                    self.model.chars_wrong = 0;

                    self.model.words_entered.clear();
                    self.model.words_tried.clear();

                    self.model.round_state = RoundState::Stopped;
                }
            }
            KeyCode::Char(c) => {
                let mut ascii_char = c;
                if event.modifiers.contains(KeyModifiers::SHIFT) {
                    ascii_char = c.to_ascii_uppercase();
                }
            
                match ascii_char {
                    ' ' => self.submit_word(),
                    _ => {
                        if self.model.round_state == RoundState::Stopped {
                            // Start the game
                            self.model.round_state = RoundState::Active;
                            self.model.start = SystemTime::now();
                        }
                            
                        self.model.word_typing.push(ascii_char);
                    }
                }
                        
            }
            _=>{}
        }
    }
    
    fn end_round(&mut self) {
        self.model.round_state = RoundState::Completed;
        self.model.word_queue.clear();
    }

    fn submit_word(&mut self){
        let mut typed_word = self.model.word_typing.chars();

        if let Some(tar_word) = self.model.word_queue.front() {
            for ch in tar_word.chars() {
                if let Some(typed_ch) = typed_word.next() {
                    if ch == typed_ch {
                        self.model.chars_correct += 1;
                    }
                    else{
                        self.model.chars_wrong += 1;
                    }
                }
            }
        }
        // There is no active word, so just clear and do nothing
        else {
            self.model.word_typing.clear();
            return;
        }


        // Save the user's attempted word
        self.model.words_entered.push(self.model.word_typing.clone());
        self.model.word_typing.clear();

        // If there was a word in the queue, save it to words_tried
        if let Some(word) = self.model.word_queue.pop_front() {
            self.model.words_tried.push(word);
        }
    }
}

#[derive(PartialEq)]
pub enum RoundState {
    Stopped,
    Active,
    Completed,
}

/// Holds all the data relevant to UI and gamestates
pub struct Model {
    /// Ring buffer of words to be entered by user
    pub word_queue:     VecDeque<String>,
    /// Characters entered by user since last space
    pub word_typing:    String,
    /// The words the user attempted to type
    pub words_tried:    Vec<String>,
    /// The words the user typed
    pub words_entered:  Vec<String>,
    pub chars_correct:  usize,
    pub chars_wrong:    usize,
    /// SystemTime at round start
    pub start:          SystemTime,
    /// Current state of round
    pub round_state:    RoundState,
    /// Dictionary file loaded to memory
    dict:               Vec<String>,
}

impl Model {
    pub fn default() -> Self {
        let dict = Self::load_dictionary();
        Self { word_queue: Self::new_word_list(&dict, 200),
               word_typing: String::new(),
               words_tried: Vec::new(),
               words_entered: Vec::new(),
               chars_correct: 0,
               chars_wrong: 0,
               start: SystemTime::now(),
               round_state: RoundState::Stopped,
               dict}
    }
    
    /// Clone a ring buffer of words using random words from a
    /// &Vec<String>.
    pub fn new_word_list(dict: &Vec<String>, num_words: usize) 
        -> VecDeque<String> {
    
        let mut rng = rand::thread_rng();
        let random_words_iter = dict.choose_multiple(&mut rng, num_words)
                                    .map(|word| word.clone());
        VecDeque::from_iter(random_words_iter)
    }

    /// Load the dictionary file into memory.
    fn load_dictionary() -> Vec<String> {
        let dict_file = File::open("dict.txt").expect("Dictionary not found.");
        let mut buf_iter = BufReader::new(dict_file).lines();
        
        // Consume buf_iter until the end of the copyright notice
        loop {
            if let Some(line) = buf_iter.next() {
                let line = line.expect("Failed to parse line");
                if line.eq("---") {
                    break;
                }
            }
        }

        // Return the lines collected into a vec
        buf_iter.map(|line| line.expect("Could not parse line."))
            .collect()

    }
}
