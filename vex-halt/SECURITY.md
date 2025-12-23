# Security Policy

## Supported Versions

We take security seriously. This project is currently in active development, and we welcome security researchers to help identify and fix security issues.

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** create public GitHub issues for security vulnerabilities
2. Email security concerns to: [security@provnai.dev](mailto:security@provnai.dev)
3. Include detailed information about the vulnerability
4. Allow reasonable time for us to respond and fix the issue before public disclosure

## Security Considerations

This benchmark evaluates AI systems for potential security issues. When running the benchmark:

- Use API keys with limited permissions
- Avoid running untrusted models or datasets
- Be aware that some test cases involve adversarial prompts
- Consider the computational and API costs before running large-scale evaluations

### Cryptographic Verification

VEX-HALT includes built-in cryptographic verification to ensure result integrity:

- **Merkle Tree Audit Trails**: Every benchmark run generates a cryptographic Merkle root
- **Tamper Detection**: Any modification to test results will change the Merkle root
- **Independent Verification**: Third parties can verify results using the published Merkle root
- **Regulatory Compliance**: Designed to meet EU AI Act requirements for audit trails

The Merkle root is displayed at the end of every benchmark run and can be used to mathematically prove that results haven't been altered.

## Responsible Disclosure

We follow responsible disclosure practices:
- Acknowledge receipt of security reports within 48 hours
- Provide regular updates on our progress
- Credit researchers for their findings (with permission)
- Publish fixes as soon as possible

Thank you for helping keep this project and the AI research community safe.