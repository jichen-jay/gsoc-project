CREATE TABLE projects (
    project_id VARCHAR(255) PRIMARY KEY,  -- url of a project repo
    project_logo VARCHAR(255),
    main_language VARCHAR(50),
    repo_stars INT NOT NULL DEFAULT 0,
    project_description TEXT,  -- description of the project, summary of its readme, etc.
    issues_list JSON,
    total_budget_allocated INT NOT NULL DEFAULT 0
) ;

CREATE TABLE issues_master (
    issue_id VARCHAR(255) PRIMARY KEY,  -- url of an issue
    node_id VARCHAR(20),
    project_id VARCHAR(255),
    project_logo VARCHAR(255),
    main_language VARCHAR(50),
    repo_stars INT NOT NULL DEFAULT 0,
    issue_title VARCHAR(255),
    issue_creator VARCHAR(50),
    issue_description TEXT,  -- description of the issue, could be truncated body text
    issue_budget INT NOT NULL DEFAULT 0,
    issue_assignees JSON,
    date_issue_assigned TIMESTAMP,
    issue_linked_pr VARCHAR(255),
    issue_status TEXT,    -- default empty, or some situation odd conditions occur
    review_status TEXT CHECK (review_status IN ('queue', 'approve', 'decline')) NOT NULL DEFAULT 'queue',
    date_approved TIMESTAMP,
    date_declined TIMESTAMP,
    issue_budget_approved BOOLEAN NOT NULL DEFAULT FALSE,
    date_budget_approved TIMESTAMP
);

CREATE TABLE issues_open (
    issue_id VARCHAR(255) PRIMARY KEY,  -- url of an issue
    node_id VARCHAR(20) ,   
    project_id VARCHAR(255) ,
    issue_creator VARCHAR(50) ,
    issue_title VARCHAR(255) ,
    issue_budget INT NOT NULL DEFAULT 0,
    issue_description TEXT 
) ;

CREATE TABLE issues_updated (
    issue_id VARCHAR(255) PRIMARY KEY,  -- url of an issue
    node_id VARCHAR(20)   
) ;


CREATE TABLE issues_repos_summarized (
    issue_or_project_id VARCHAR(255) PRIMARY KEY, -- url of an issue
    issue_or_project_summary TEXT,
    keyword_tags JSONB,
    keyword_tags_text TEXT GENERATED ALWAYS AS (keyword_tags::text) STORED,
    indexed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE issues_assign_comment (
    comment_id SERIAL PRIMARY KEY,  -- id of a comment
    issue_id VARCHAR(255),  -- url of an issue
    node_id VARCHAR(20),
    issue_assignees JSON,
    comment_creator VARCHAR(50) NOT NULL,
    comment_date TIMESTAMP NOT NULL,  -- date of the comment
    comment_body TEXT NOT NULL  -- content of the comment
);

CREATE TABLE issues_closed (
    issue_id VARCHAR(255) PRIMARY KEY,  -- url of an issue
    issue_assignees JSON,    
    issue_linked_pr VARCHAR(255) 
) ;

CREATE TABLE pull_requests (
    pull_id VARCHAR(255) PRIMARY KEY,  -- url of pull_request
    pull_title VARCHAR(255),
    pull_author VARCHAR(50),
    project_id VARCHAR(255),
    date_merged TIMESTAMP  -- date and time when the pull request was merged
);