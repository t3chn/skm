use std::path::Path;
use git2::{Repository, StatusOptions};
use chrono::{DateTime, Utc, TimeZone};
use crate::{Result, GitStatus};

/// Get Git repository status for a project
pub fn get_git_status(project_path: &Path) -> Result<GitStatus> {
    let repo = match Repository::open(project_path) {
        Ok(repo) => repo,
        Err(_) => {
            // Not a git repository
            return Ok(GitStatus {
                is_repo: false,
                branch: None,
                clean: true,
                last_commit: None,
                ahead: 0,
                behind: 0,
            });
        }
    };
    
    let is_repo = true;
    let branch = get_current_branch(&repo)?;
    let clean = is_working_tree_clean(&repo)?;
    let last_commit = get_last_commit_time(&repo)?;
    let (ahead, behind) = get_ahead_behind(&repo)?;
    
    Ok(GitStatus {
        is_repo,
        branch,
        clean,
        last_commit,
        ahead,
        behind,
    })
}

fn get_current_branch(repo: &Repository) -> Result<Option<String>> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return Ok(None), // Detached HEAD or no commits
    };
    
    if let Some(name) = head.shorthand() {
        Ok(Some(name.to_string()))
    } else {
        Ok(None)
    }
}

fn is_working_tree_clean(repo: &Repository) -> Result<bool> {
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);
    
    let statuses = repo.statuses(Some(&mut status_opts))?;
    Ok(statuses.is_empty())
}

fn get_last_commit_time(repo: &Repository) -> Result<Option<DateTime<Utc>>> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return Ok(None),
    };
    
    let oid = match head.target() {
        Some(oid) => oid,
        None => return Ok(None),
    };
    
    let commit = repo.find_commit(oid)?;
    let time = commit.time();
    let timestamp = time.seconds();
    
    Ok(Some(Utc.timestamp_opt(timestamp, 0).unwrap()))
}

fn get_ahead_behind(repo: &Repository) -> Result<(u32, u32)> {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return Ok((0, 0)),
    };
    
    let local_oid = match head.target() {
        Some(oid) => oid,
        None => return Ok((0, 0)),
    };
    
    // Try to find the upstream branch
    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return Ok((0, 0)),
    };
    
    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(branch) => branch,
        Err(_) => return Ok((0, 0)),
    };
    
    let upstream = match branch.upstream() {
        Ok(upstream) => upstream,
        Err(_) => return Ok((0, 0)), // No upstream configured
    };
    
    let upstream_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return Ok((0, 0)),
    };
    
    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;
    Ok((ahead as u32, behind as u32))
}

/// Check if there are any error markers in recent commits
pub fn has_recent_errors(path: &Path) -> Result<bool> {
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(_) => return Ok(false), // Not a git repo
    };
    
    Ok(has_recent_errors_in_repo(&repo))
}

fn has_recent_errors_in_repo(repo: &Repository) -> bool {
    let error_markers = ["FIXME", "TODO", "XXX", "HACK", "BUG"];
    
    // Check the last 5 commits for error markers
    if let Ok(mut revwalk) = repo.revwalk() {
        let _ = revwalk.push_head();
        let _ = revwalk.set_sorting(git2::Sort::TIME);
        
        for (i, oid) in revwalk.enumerate() {
            if i >= 5 {
                break;
            }
            
            if let Ok(oid) = oid {
                if let Ok(commit) = repo.find_commit(oid) {
                    if let Some(message) = commit.message() {
                        for marker in &error_markers {
                            if message.contains(marker) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    
    false
}