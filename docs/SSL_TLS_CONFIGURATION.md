# SSL/TLS Configuration for PostgreSQL

This document describes how to configure SSL/TLS connections for PostgreSQL database in VK Teams Bot.

## Overview

Starting from version 0.11.0, VK Teams Bot supports SSL/TLS encryption for PostgreSQL connections. This feature enhances security by encrypting data in transit between your application and the database.

## Configuration

### Basic SSL Configuration

Add the SSL configuration to your `storage.database` section:

```toml
[storage.database]
url = "postgresql://user:password@host:5432/database"
max_connections = 20
connection_timeout = 30
auto_migrate = true

[storage.database.ssl]
enabled = true          # Enable SSL/TLS
mode = "require"        # SSL mode
```

### SSL Modes

The `mode` field supports the following PostgreSQL SSL modes:

| Mode | Description |
|------|-------------|
| `disable` | No SSL connection |
| `prefer` | Try SSL first, fall back to non-SSL if unavailable (default) |
| `require` | Require SSL connection, but don't verify certificates |
| `verify-ca` | Require SSL and verify that the server certificate is issued by a trusted CA |
| `verify-full` | Require SSL, verify CA, and verify that the server hostname matches the certificate |

### Certificate Configuration

For enhanced security with certificate verification:

```toml
[storage.database.ssl]
enabled = true
mode = "verify-full"
root_cert = "/path/to/ca-cert.pem"        # Root CA certificate
client_cert = "/path/to/client-cert.pem"  # Client certificate (optional)
client_key = "/path/to/client-key.pem"    # Client private key (optional)
```

## Examples

### Local Development (No SSL)

```toml
[storage.database.ssl]
enabled = false
```

### Production with Basic SSL

```toml
[storage.database.ssl]
enabled = true
mode = "require"
```

### Production with Full Verification

```toml
[storage.database.ssl]
enabled = true
mode = "verify-full"
root_cert = "/etc/ssl/certs/postgres-ca.pem"
```

### Cloud Providers

#### AWS RDS

```toml
[storage.database]
url = "postgresql://user:password@myinstance.region.rds.amazonaws.com:5432/mydb"

[storage.database.ssl]
enabled = true
mode = "verify-full"
root_cert = "/opt/aws-rds-ca-cert.pem"  # Download from AWS
```

#### Google Cloud SQL

```toml
[storage.database]
url = "postgresql://user:password@ip-address:5432/mydb"

[storage.database.ssl]
enabled = true
mode = "verify-full"
root_cert = "/path/to/server-ca.pem"
client_cert = "/path/to/client-cert.pem"
client_key = "/path/to/client-key.pem"
```

#### Azure Database for PostgreSQL

```toml
[storage.database]
url = "postgresql://user@servername:password@servername.postgres.database.azure.com:5432/mydb"

[storage.database.ssl]
enabled = true
mode = "verify-full"
root_cert = "/path/to/DigiCertGlobalRootCA.crt.pem"  # Download from Azure
```

## Certificate Management

### Obtaining Certificates

1. **Root CA Certificate**: Download from your database provider
   - AWS RDS: https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/UsingWithRDS.SSL.html
   - Google Cloud: From Cloud Console > SQL > Instance > Connections > SSL
   - Azure: https://docs.microsoft.com/en-us/azure/postgresql/concepts-ssl-connection-security

2. **Client Certificates**: Generate through your database provider's console or API

### File Permissions

Ensure proper permissions for certificate files:

```bash
chmod 600 /path/to/client-key.pem  # Private key must be readable only by owner
chmod 644 /path/to/client-cert.pem # Certificate can be readable by all
chmod 644 /path/to/ca-cert.pem     # CA certificate can be readable by all
```

## Connection String Parameters

Alternatively, you can configure SSL directly in the connection URL:

```toml
[storage.database]
# With SSL mode
url = "postgresql://user:password@host:5432/database?sslmode=require"

# With certificates (URL-encoded paths)
url = "postgresql://user:password@host:5432/database?sslmode=verify-full&sslrootcert=/path/to/ca.pem"
```

Note: When using both URL parameters and configuration file settings, the configuration file takes precedence.

## Troubleshooting

### Common Issues

1. **Certificate not found**
   - Check file paths are absolute
   - Verify file permissions
   - Ensure the application has read access

2. **SSL connection failed**
   - Verify the database server supports SSL
   - Check firewall rules allow SSL port (usually 5432)
   - Confirm SSL mode compatibility

3. **Certificate verification failed**
   - Ensure the hostname matches the certificate
   - Verify the CA certificate is correct
   - Check certificate validity dates

### Debug Logging

Enable debug logging to troubleshoot SSL issues:

```bash
RUST_LOG=sqlx=debug cargo run
```

## Security Best Practices

1. **Always use SSL in production** - At minimum, use `mode = "require"`
2. **Verify certificates** - Use `verify-ca` or `verify-full` when possible
3. **Protect private keys** - Set file permissions to 600
4. **Rotate certificates** - Follow your organization's certificate rotation policy
5. **Use environment variables** - For sensitive data like passwords:

```toml
[storage.database]
url = "${DATABASE_URL}"  # Set via environment variable
```

## Performance Considerations

SSL/TLS adds some overhead to database connections:
- Initial connection establishment is slower due to handshake
- Slight increase in CPU usage for encryption/decryption
- Minimal impact on query performance for most applications

The security benefits typically outweigh the performance costs in production environments.

## See Also

- [PostgreSQL SSL Support Documentation](https://www.postgresql.org/docs/current/ssl-tcp.html)
- [Storage Configuration Guide](../examples/vkteams-bot-config.toml)
- [VK Teams Bot Storage Documentation](../crates/vkteams-bot/src/storage/README.md)