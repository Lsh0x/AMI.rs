# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take the security of AMI.rs seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Please do NOT:

- Open a public GitHub issue
- Discuss the vulnerability in public forums or social media

### Please DO:

1. **Email us directly** at: github@lsh.tech
2. **Include the following information**:
   - Type of vulnerability (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
   - Full paths of source file(s) related to the vulnerability
   - The location of the affected source code (tag/branch/commit or direct URL)
   - Any special configuration required to reproduce the issue
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact of the issue, including how an attacker might exploit it

### What to Expect:

- We will acknowledge receipt of your vulnerability report within 48 hours
- We will send you regular updates about our progress
- We will notify you when the vulnerability is fixed
- We will publicly disclose the vulnerability in a responsible manner after a fix is released

### Security Updates

Security updates will be released as patch versions and will be clearly marked in the CHANGELOG.md file.

### Security Best Practices

When using AMI.rs, we recommend:

1. **Keep dependencies up to date**: Regularly update to the latest version
2. **Use secure credentials**: Never hardcode AWS credentials in your code
3. **Follow least privilege**: Grant only the minimum required permissions
4. **Enable MFA**: Use multi-factor authentication for sensitive operations
5. **Monitor access**: Regularly audit IAM access logs

## Attribution

We appreciate the security research community and will acknowledge researchers who responsibly disclose vulnerabilities to us (unless you prefer to remain anonymous).

## Contact

For any security-related questions or concerns, please contact: github@lsh.tech

