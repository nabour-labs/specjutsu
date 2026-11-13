## Requirements

### Requirement: Login
#### Scenario: AUTH-001 Log in with a valid password
- **WHEN** the user submits valid credentials
- **THEN** a session is created

#### Scenario: AUTH-002 Reject a wrong password
- **WHEN** the user submits a wrong password
- **THEN** the response is 401

#### Scenario: Lock account after five failures
- **WHEN** five logins fail
- **THEN** the account is locked

```markdown
#### Scenario: AUTH-777 Example inside a fence is ignored
```
