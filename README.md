# Local
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/CN=127.0.0.1" -addext "subjectAltName=IP:127.0.0.1"


# Prod
## Install Certbot

```
$ sudo snap install --classic certbot
$ sudo ln -s /snap/bin/certbot /usr/bin/certbot
$ sudo certbot certonly --manual --preferred-challenges dns -d your_domain.com

```

Follow Certbot's instructions. You will be prompted to create a DNS TXT record for validation.<br/><br/>
Access your DNS manager and add the TXT record provided by Certbot.<br/><br/>
Once the DNS record has been added, continue the process with Certbot. Certbot will check the TXT record and, if everything is correct, generate the certificate.

```
$ sudo chown your_user:your_user_group /etc/letsencrypt/live/your_domain.com/*.pem
$ sudo chmod 640 /etc/letsencrypt/live/your_domain.com/*.pem
```
