import random

# Generate 100K diverse URLs for testing
domains = ["example.com", "test.org", "site.net"]
paths = ["page", "article", "post", "item"]
tracking = ["utm_source=google", "fbclid=123", ""]

with open("tests/fixtures/urls_100k.txt", "w") as f:
    for i in range(100000):
        domain = random.choice(domains)
        path = random.choice(paths)
        tracking_param = random.choice(tracking)

        url = f"https://{domain}/{path}{i}"
        if tracking_param:
            url += f"?{tracking_param}"

        f.write(url + "\n")

# Generate duplicates (20%)
with open("tests/fixtures/urls_with_duplicates.txt", "w") as f:
    urls = open("tests/fixtures/urls_100k.txt").readlines()
    for url in urls:
        f.write(url)
        if random.random() < 0.2:
            f.write(url.strip() + "&ref=copy\n")
