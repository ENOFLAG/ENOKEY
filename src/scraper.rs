use error::EnokeysError;

pub fn fetch(user: &str, provider: &str) -> Result<Vec<String>, EnokeysError> {
    match provider {
        "github" => fetch_github(user),
        "gitlab" => fetch_gitlab(user),
        "tublab" => fetch_tublab(user),
        "enolab" => fetch_enolab(user),
        x => Err(EnokeysError::InvalidProviderError(x.to_string()))
    }
}

fn fetch_github(user: &str) -> Result<Vec<String>, EnokeysError> {
    let mut res = reqwest::get(&format!("https://www.github.com/{}.keys",user))?;
    assert_eq!(res.status(),200);
    Ok(res.text().unwrap().split('\n').filter(|&i|!i.is_empty()).map(|s|format!("{} {}@github", s, user)).collect::<Vec<String>>())
}

fn fetch_gitlab(user: &str) -> Result<Vec<String>, EnokeysError> {
    let mut res = reqwest::get(&format!("https://www.gitlab.com/{}.keys",user))?;
    assert_eq!(res.status(),200);
    Ok(res.text().unwrap().split('\n').filter(|&i|!i.is_empty()).map(|s|format!("{} {}@gitlab", s, user)).collect::<Vec<String>>())
}

fn fetch_tublab(user: &str) -> Result<Vec<String>, EnokeysError> {
    let mut res = reqwest::get(&format!("https://gitlab.tubit.tu-berlin.de/{}.keys",user))?;
    assert_eq!(res.status(),200);
    Ok(res.text().unwrap().split('\n').filter(|&i|!i.is_empty()).map(|s|format!("{} {}@tublab", s, user)).collect::<Vec<String>>())
}

fn fetch_enolab(user: &str) -> Result<Vec<String>, EnokeysError> {
    let mut res = reqwest::get(&format!("https://gitlab.enoflag.de/{}.keys",user))?;
    assert_eq!(res.status(),200);
    Ok(res.text().unwrap().split('\n').filter(|&i|!i.is_empty()).map(|s|format!("{} {}@enolab", s, user)).collect::<Vec<String>>())
}
