/// Handler holds all required information for handling authentication callbacks from `git2`.
pub(crate) struct AuthHandler {
    config: git2::Config,
    /// Shows if the `AuthHandler` tried to make `SSH` authentication so far.
    ssh_authentication_attempt: bool,
    /// Shows if the `AuthHandler` tried to make `USER_PASS_PLAINTEXT` authentication so far.
    plain_user_pass_attempt: bool,
}

impl AuthHandler {
    /// Creates a new `AuthHandler` from all fields of the struct. If there are no specific reasons
    /// not to, `default_with_config` should be prefered.
    fn new(
        config: git2::Config,
        ssh_authentication_attempt: bool,
        plain_user_pass_attempt: bool,
    ) -> Self {
        Self {
            config,
            ssh_authentication_attempt,
            plain_user_pass_attempt,
        }
    }

    /// Creates a new `AuthContext` with provided `git2::Config` and default values for other
    /// context used during handling authentication callbacks.
    pub(crate) fn default_with_config(config: git2::Config) -> Self {
        let ssh_authentication_attempt = false;
        let plain_user_pass_attempt = false;
        Self::new(config, ssh_authentication_attempt, plain_user_pass_attempt)
    }

    pub fn handle_callback(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: git2::CredentialType,
    ) -> Result<git2::Cred, git2::Error> {
        if allowed.contains(git2::CredentialType::SSH_KEY) && !self.ssh_authentication_attempt {
            self.ssh_authentication_attempt = true;
            // If SSH_KEY authentication is allowed, a callback username is provided, so the
            // following unwrap is guaranteed.
            let username = username.ok_or_else(|| {
                git2::Error::from_str("username must be provided with SSH_KEY callback")
            })?;
            return git2::Cred::ssh_key_from_agent(username);
        }
        if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT)
            && !self.plain_user_pass_attempt
        {
            self.plain_user_pass_attempt = true;
            return git2::Cred::credential_helper(&self.config, url, username);
        }
        if allowed.contains(git2::CredentialType::DEFAULT) {
            return git2::Cred::default();
        }
        Err(git2::Error::from_str(
            "Tried all possible credential types for authentication",
        ))
    }
}
