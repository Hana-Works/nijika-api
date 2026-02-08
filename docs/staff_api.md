# Staff API Reference

This document describes the administrative endpoints available for moderators and administrators in the `nijika-api`. These endpoints are primarily used through the web interface.

## Access Control

- **Moderator**: Can view users, detailed user info, and audit logs. Can toggle user status (activate/deactivate).
- **Administrator**: Can do everything a moderator can, plus adjust user credits, change user roles, and view system-wide status.

## Endpoints

### User Management

#### List Users
Returns a paginated list of all registered users.

- **URL:** `/staff/users`
- **Method:** `GET`
- **Role:** Moderator
- **Query Parameters:**
    - `page` (optional): Page number (default: 1).

#### User Details
Returns detailed information about a specific user, including recent usage logs and transactions.

- **URL:** `/staff/users/{id}`
- **Method:** `GET`
- **Role:** Moderator

#### Toggle User Status
Activates or deactivates a user account. Deactivated users cannot use the API or log in.

- **URL:** `/staff/users/{id}/toggle-status`
- **Method:** `POST`
- **Role:** Moderator

#### Adjust Credits
Adds or subtracts credits from a user's account.

- **URL:** `/staff/users/{id}/adjust-credits`
- **Method:** `POST`
- **Role:** Administrator
- **Form Data:**
    - `amount` (decimal): The amount to add (positive) or subtract (negative).
    - `reason` (string): Description of the adjustment.

#### Change User Role
Changes the authorization level of a user.

- **URL:** `/staff/users/{id}/change-role`
- **Method:** `POST`
- **Role:** Administrator
- **Form Data:**
    - `role` (string): One of `admin`, `moderator`, `user`.

### System & Auditing

#### Audit Logs
Returns a paginated list of all service usage logs across all users.

- **URL:** `/staff/logs`
- **Method:** `GET`
- **Role:** Moderator
- **Query Parameters:**
    - `page` (optional): Page number (default: 1).

#### System Status
Provides high-level system metrics, including total users, total credits in circulation, and overall success rate.

- **URL:** `/staff/system`
- **Method:** `GET`
- **Role:** Administrator
