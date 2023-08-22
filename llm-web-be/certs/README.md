# Certificates to manage TLS

It is possible to create a Root CA, install the Root CA certificate on the browser, generate a private public key, and use the Root CA to create a certificate for the server.  Or just a self signed certificate and install an exception for the browser....

## The Private Key

This is the secret for the server known only to the server.

`openssl genrsa -out private4096.key 4096 `

* `genrsq`  Generating an RSA key
* `-out private.key` The output file for the key
* `4096` The number of bits

## Creating the Certificate Signing Request (CSR)

Skip this section

This is the document that is used to generate the server's public certificate

`openssl req -key private4096.key -new -out certificate.csr`

* `req` A certificate request (or signing)
* `-key private4096.key` The server's secret
* `-nw`  Creating a CSR
* `certificate.csr` The name of the resulting file

There is no need to enter anything but defaults.

### Extensions

If this CSR is to be sent to a Root CA for signing, and then is to be accepted by browsers it is important to add x509 extensions, specifically an `Alternative Name` section with a `DNS` entry for the URL of the server to be protected.

## Generating the Self-Signed Certificate

The "self-signed" certificate is signed with the server's private key, as opposed to one signed with a Root CA's private key.

We do not need the `CSR` to do this:

`openssl req -new -x509 -key private4096.key -days 3650 -out public.crt`

* `req` Creating, or inspecting a certificate or a certificate signing request
* `-new`  Generating a new certificate or request
* `-x509` Output a self-signed certificate (not a request)
* `-days 3650` How long the certificate is valid for.  (Some domains, particularly Apple, have limts on this) 
* `-out public.crt` Output file
