#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLink {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedLink {
    pub raw: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutcome {
    pub accepted: Vec<ParsedLink>,
    pub rejected: Vec<RejectedLink>,
}

pub fn parse_paste(input: &str) -> ParseOutcome {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    for token in input.split_whitespace() {
        match parse_one(token) {
            Ok(link) => accepted.push(link),
            Err(reason) => rejected.push(RejectedLink {
                raw: token.to_owned(),
                reason,
            }),
        }
    }
    ParseOutcome { accepted, rejected }
}

fn parse_one(raw: &str) -> Result<ParsedLink, String> {
    let invalid = || format!("Tautan tidak valid, {raw}");
    let without_fragment = raw.split_once('#').map(|(head, _)| head).unwrap_or(raw);
    let without_query = without_fragment
        .split_once('?')
        .map(|(head, _)| head)
        .unwrap_or(without_fragment);
    let rest = strip_scheme(without_query).map_err(|_| invalid())?;
    let (host, path) = rest.split_once('/').ok_or_else(invalid)?;
    if !host.eq_ignore_ascii_case("github.com") && !host.eq_ignore_ascii_case("www.github.com") {
        return Err(invalid());
    }
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.len() < 2 {
        return Err(invalid());
    }
    let owner = segments[0];
    let mut repo = segments[1];
    if let Some(stripped) = repo.strip_suffix(".git") {
        repo = stripped;
    }
    if !is_name(owner) || !is_name(repo) {
        return Err(invalid());
    }
    let git_ref = match &segments[2..] {
        [] => None,
        ["tree", rest @ ..]
            if !rest.is_empty() && rest.iter().all(|segment| is_ref_segment(segment)) =>
        {
            Some(rest.join("/"))
        }
        _ => return Err(invalid()),
    };
    Ok(ParsedLink {
        owner: owner.to_owned(),
        repo: repo.to_owned(),
        git_ref,
        raw: raw.to_owned(),
    })
}

fn strip_scheme(value: &str) -> Result<&str, ()> {
    if let Some(idx) = value.find("://") {
        let scheme = &value[..idx];
        if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") {
            return Ok(&value[idx + 3..]);
        }
        return Err(());
    }
    if value.starts_with("git@") {
        return Err(());
    }
    Ok(value)
}

pub(crate) fn is_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.')
        && !value.ends_with('.')
        && value != "."
        && value != ".."
}

pub(crate) fn is_ref_segment(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && value.chars().all(|ch| !ch.is_control())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_owner_repo_with_or_without_scheme_and_git_suffix() {
        for raw in [
            "github.com/octocat/hello",
            "https://github.com/octocat/hello",
            "http://github.com/octocat/hello",
            "https://www.github.com/octocat/hello",
            "https://github.com/octocat/hello.git",
            "https://github.com/octocat/hello/",
        ] {
            let parsed = parse_paste(raw);
            assert!(parsed.rejected.is_empty(), "{raw}");
            assert_eq!(parsed.accepted.len(), 1, "{raw}");
            assert_eq!(parsed.accepted[0].owner, "octocat");
            assert_eq!(parsed.accepted[0].repo, "hello");
            assert_eq!(parsed.accepted[0].git_ref, None);
        }
        let parsed = parse_paste("HTTPS://GitHub.com/OctoCat/Hello-World");
        assert!(parsed.rejected.is_empty());
        assert_eq!(parsed.accepted[0].owner, "OctoCat");
        assert_eq!(parsed.accepted[0].repo, "Hello-World");
        assert_eq!(parsed.accepted[0].git_ref, None);
    }

    #[test]
    fn accepts_tree_ref_including_slashes() {
        let parsed = parse_paste("https://github.com/octocat/hello/tree/feature/one");
        assert!(parsed.rejected.is_empty());
        assert_eq!(parsed.accepted[0].git_ref.as_deref(), Some("feature/one"));
        assert_eq!(parsed.accepted[0].repo, "hello");
    }

    #[test]
    fn splits_a_paste_and_keeps_valid_links_beside_junk() {
        let parsed = parse_paste(
            "https://github.com/octocat/hello\nhttps://gist.github.com/octocat/1 not a link\nhttps://github.com/octocat/world.git",
        );
        assert_eq!(
            parsed
                .accepted
                .iter()
                .map(|link| link.repo.as_str())
                .collect::<Vec<_>>(),
            vec!["hello", "world"]
        );
        assert_eq!(parsed.rejected.len(), 4);
        assert!(parsed.rejected[0].reason.contains("gist.github.com"));
        assert!(
            parsed
                .rejected
                .iter()
                .all(|item| item.reason.starts_with("Tautan tidak valid"))
        );
        assert!(parsed.rejected.iter().any(|item| item.raw == "not"));
    }

    #[test]
    fn rejects_path_escape_and_owner_only_issues_other_hosts_and_ssh() {
        for raw in [
            "https://github.com/octocat",
            "https://github.com/octocat/hello/issues/1",
            "https://gitlab.com/octocat/hello",
            "git@github.com:octocat/hello.git",
            "https://github.com/octocat/hello/blob/main/README.md",
            "https://github.com/../hello",
            "https://github.com/octocat/..",
            "https://github.com/octocat/hello/tree/..",
            "https://github.com/octocat/hello%2F..",
        ] {
            let parsed = parse_paste(raw);
            assert!(parsed.accepted.is_empty(), "{raw}");
            assert_eq!(parsed.rejected.len(), 1, "{raw}");
            assert!(
                parsed.rejected[0].reason.starts_with("Tautan tidak valid"),
                "{raw}"
            );
        }
    }
}
