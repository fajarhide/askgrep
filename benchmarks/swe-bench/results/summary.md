# SWE-bench Lite retrieval, 9 instances

Issue text truncated to 400 characters. 18 minutes of wall clock.

| | recall@1 | recall@5 | recall@10 | MRR |
| --- | --- | --- | --- | --- |
| askgrep | 0.22 | 0.33 | 0.56 | 0.294 |
| bm25 | 0.11 | 0.56 | 1.00 | 0.332 |

| instance | gold | askgrep rank | bm25 rank | chunks | s |
| --- | --- | --- | --- | --- | --- |
| pallets__flask-4045 | src/flask/blueprints.py | 89 | 8 | 6893 | 112 |
| pallets__flask-4992 | src/flask/config.py | 54 | 1 | 6915 | 98 |
| pallets__flask-5063 | src/flask/cli.py | 1 | 9 | 6899 | 101 |
| psf__requests-1963 | requests/sessions.py | 10 | 4 | 9218 | 1 |
| psf__requests-2148 | requests/models.py | 1 | 2 | 9368 | 147 |
| psf__requests-2317 | requests/sessions.py | 10 | 6 | 9449 | 142 |
| psf__requests-2674 | requests/adapters.py | 3 | 3 | 9751 | 156 |
| psf__requests-3362 | requests/utils.py | 19 | 6 | 10468 | 168 |
| psf__requests-863 | requests/models.py | 30 | 3 | 11758 | 126 |

askgrep ranked the gold file higher on 2 of 9, tied on 1, lower on 6.
