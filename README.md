# GitHub Self-Hosted Runner Status Monitor
Monitor Uptime Status of GitHub Self-Hosted Runner.
This project is designed to be run inside of a Docker container but does support native execution.

The monitor pings the GitHub API to request the current status of all self-hosted runners.
Whenever a runner
- goes offline,
- comes back online,
- has been removed or
- has been created
an alert is sent to a Webhook.

The pings are initiated via the Unix SIGHUP signal.
A different process (usually cron) sends a SIGHUP to gh_runner_uptime every user specified interval.

## Deployment
### Docker-Compose
See the [example docker-compose.yml](./example_deployment/docker-compose.yml).

### Configuration
gh_runner_uptime is configured using a yaml file.
This file needs to be in the current working directory of the gh_runner_uptime process and must be called `config.yaml`.
See the [example config.yaml](./example_deployment/config.yaml).

### Alert Templates
Whenever one of the four types of alert occur an HTML POST request is sent to the Webhook.
You can define what gets sent for each case.
gh_runner_uptime uses [Tera](https://keats.github.io/tera) as a template engine which is similar to Jinja2.
Your templates have access to an `old_runner` and/or `new_runner` object of type `Runner` the definition of which is in `src/structs.rs`.

See [test_online_template.txt.j2](./src/tests/test_online_template.txt.j2) for an example.

## GitHub PAT
You need to create a token to authorize gh_runner_uptime's GitHub API access.
These are some but not all ways of creating such a token.

### Fine Grained Token (for Organization)
If gh_runner_uptime should monitor an organisation or a repository owned by an organisation you need to [set a personal access token policy for your organization](https://docs.github.com/en/enterprise-server@3.12/organizations/managing-programmatic-access-to-your-organization/setting-a-personal-access-token-policy-for-your-organization).
Once you have done that you can create a fine grained token **for the organization**.
Don't create a token for a user in this case.
The fine grained token needs this permission:
```
"Self-hosted runners" organization permissions (read)
```
See [the docs](https://docs.github.com/en/enterprise-server@3.12/rest/actions/self-hosted-runners?apiVersion=2022-11-28#list-self-hosted-runners-for-an-organization--fine-grained-access-tokens) for more.

### Fine Grained Token (for Repository)
Create a fine grained token with this permission:
```
"Administration" repository permissions (read)
```
See [the docs](https://docs.github.com/en/enterprise-server@3.12/rest/actions/self-hosted-runners?apiVersion=2022-11-28#list-self-hosted-runners-for-a-repository--fine-grained-access-tokens) for more.

<!-- TODO: enterprise fine grained tokens -->

### Classic Token
Create a classic personal access token for a user with access to the organisation with these permissions:
- repo (needed only for repo runner)
- manage_runners:org (needed only for org runner)
<!-- TODO: add -->
<!-- - manage_runners:enterprise  (needed only for enterprise runner) -->

## Building and Testing
Simply run `cargo build` to build on your system.
Run `cargo test` to run the unit tests.
The docker image can be built with `docker build -t gh_runner_uptime .`.
