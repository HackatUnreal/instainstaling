use std::fmt;

use ureq::serde_json::{from_str, Value};
use ureq::Agent;

pub struct Insta {
    pub words_list: Vec<Word>,
    pub username: String,
    pub password: String,
    pub child_id: String,

    pub agent: Agent,
}

impl Insta {
    pub fn generate_word(&self) -> Result<Word, &'static str> {
        let word_form = [("child_id", self.child_id.as_str())];

        let word_request = self.agent.post("https://instaling.pl/ling2/server/actions/generate_next_word.php")
            .send_form(&word_form).unwrap();
        
        let word_response: Value = from_str(&word_request.into_string().unwrap()).unwrap();
        let word_id = word_response["id"].as_str();

        if word_id == None {
            return Err("finished");
        }

        let word_id = word_id.unwrap();

        return Ok(Word::new(word_id.to_string(), String::new()));
    } 

    pub fn generate_answer(&self, word: &mut Word) {
        let answer_request = self.agent.get("https://instaling.pl/ling2/server/actions/getAudioUrl.php")
            .query("id", &word.id)
            .send_string("").unwrap();

        let answer_response: Value = from_str(&answer_request.into_string().unwrap()).unwrap();
        let answer_url = answer_response["url"].as_str().unwrap();

        word.parse(answer_url, &self.words_list);
    }

    pub fn check_answer(&mut self, word: &Word) -> AnswerResult {
        let check_form = [("child_id", self.child_id.as_str()), ("word_id", word.id.as_str()), ("answer", word.answer.as_str()), ("version", "C65E24B29F60B1221EC23D979C9707D2")];

        let check_request = self.agent.post("https://instaling.pl/ling2/server/actions/save_answer.php")
            .send_form(&check_form).unwrap();

        let check_response: Value = from_str(&check_request.into_string().unwrap()).unwrap();
        let check_word = check_response["answershow"].as_str();

        if check_word.is_none() {
            println!("{:?}", &check_response);
            return AnswerResult::Error; 
        }

        let check_word = check_word.unwrap();
        let answer = check_word == word.answer;


        if answer {
            return AnswerResult::Good; 
        } else {
            let mut word_copy = word.clone();
            word_copy.answer = check_word.to_string();
            self.words_list.push(word_copy);

            return AnswerResult::Bad; 
        }
    }


}
#[derive(PartialEq)]
pub enum InstaError {
    WrongCreds
}


impl fmt::Debug for InstaError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error lol")
    }
}



#[derive(PartialEq)]
pub enum AnswerResult {
    Good,
    Bad,
    Error
}

pub struct InstaBuilder {
    pub username: String,
    pub password: String,
    pub child_id: String,

    pub agent: Agent,

}

impl InstaBuilder {
    pub fn new(agent: Agent) -> Self {
        let username = String::new();
        let password = String::new();
        let child_id = String::new();
        Self {username, password, child_id, agent}
    }

    pub fn credentials(mut self, username: &str, password: &str) -> Self {
        self.username = username.to_string();
        self.password = password.to_string();

        return self;
    }

    pub fn login(self) -> Self {
        // create the params 
        let login_params = [("action", "login"), ("from", ""), ("log_email", self.username.as_str()), ("log_password", self.password.as_str())];

        self.agent.post("https://instaling.pl/teacher.php?page=teacherActions")
            .send_form(&login_params).unwrap();

        return self;
    }

    pub fn start_session(self) -> Self {
        // create the params
        let sess_params = [("child_id", self.child_id.as_str())];

        self.agent.post("https://instaling.pl/ling2/server/actions/init_session.php")
            .send_form(&sess_params).unwrap();

        return self;
    }

   
    pub fn get_child_id(mut self) -> Result<Self, InstaError> {
        let disp_request = self.agent.get("https://instaling.pl/learning/dispatcher.php?from=")
            .send_string("").unwrap();

        if disp_request.get_url().contains("expired") {
            return Err(InstaError::WrongCreds);
        }


        self.child_id = disp_request.get_url().split_at(59).1.to_string();
        Ok(self)
    }

    pub fn build(self) -> Insta {
        let words_list: Vec<Word> = vec![];
        let username = self.username;
        let password = self.password;
        let child_id = self.child_id;
        
        let agent = self.agent;
        Insta {words_list, username, password, child_id, agent}
    }
}

#[derive(Clone)]
pub struct Word {
    pub id: String,
    pub answer: String 
}

impl Word {
    pub fn new(id: String, answer: String) -> Self {
        Self {id, answer}
    }
    
    pub fn parse(&mut self, raw_url: &str, words_saved: &Vec<Self>) {
        // hacky way of getting only the word
        // the first 28 chars is the path
        let mut answer = raw_url.split_at(28).1.replace(".mp3", "");

        // parsing rules
        for word in words_saved {
            if word.id == self.id {
                answer = word.answer.clone();
            }
        }
        
        self.answer = answer;
    }
}