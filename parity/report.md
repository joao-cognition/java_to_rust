# Java ↔ Rust parity report

- java: `http://localhost:9966/travel/api`
- rust: `http://localhost:9966/travel/api`
- mode: record
- schedule first day: 2026-11-05
- generated: 2026-11-04T13:09:00.752790533+00:00
- cases: 843
- verdict: **IDENTICAL**

Each case must return the same HTTP status, `Content-Type` and body (key order ignored; values, array order and number text compared exactly; problem bodies compared field by field).

| endpoint | cases | identical | diff | rust down | java down |
|---|---:|---:|---:|---:|---:|
| `airports` | 1 | 1 | 0 | 0 | 0 |
| `airports/{iata}` | 22 | 22 | 0 | 0 | 0 |
| `airlines` | 1 | 1 | 0 | 0 | 0 |
| `airlines/{iata}` | 15 | 15 | 0 | 0 | 0 |
| `flights` | 90 | 90 | 0 | 0 | 0 |
| `flights/{id}` | 10 | 10 | 0 | 0 | 0 |
| `offers` | 421 | 421 | 0 | 0 | 0 |
| `travelers` | 35 | 35 | 0 | 0 | 0 |
| `travelers/{id}` | 7 | 7 | 0 | 0 | 0 |
| `bookings` | 35 | 35 | 0 | 0 | 0 |
| `bookings/{recordLocator}` | 204 | 204 | 0 | 0 | 0 |
| `unknown route` | 2 | 2 | 0 | 0 | 0 |
| **all** | **843** | **843** | **0** | **0** | **0** |

## Cases

| # | endpoint | case | path | java | rust | content type (java) | verdict | differences |
|---:|---|---|---|---:|---:|---|---|---|
| 1 | `airports` | airports list | `airports` | 200 | 200 | `application/json` | identical |  |
| 2 | `airports/{iata}` | airport NCE | `airports/NCE` | 200 | 200 | `application/json` | identical |  |
| 3 | `airports/{iata}` | airport LHR | `airports/LHR` | 200 | 200 | `application/json` | identical |  |
| 4 | `airports/{iata}` | airport CDG | `airports/CDG` | 200 | 200 | `application/json` | identical |  |
| 5 | `airports/{iata}` | airport JFK | `airports/JFK` | 200 | 200 | `application/json` | identical |  |
| 6 | `airports/{iata}` | airport SIN | `airports/SIN` | 200 | 200 | `application/json` | identical |  |
| 7 | `airports/{iata}` | airport MAD | `airports/MAD` | 200 | 200 | `application/json` | identical |  |
| 8 | `airports/{iata}` | airport FRA | `airports/FRA` | 200 | 200 | `application/json` | identical |  |
| 9 | `airports/{iata}` | airport AMS | `airports/AMS` | 200 | 200 | `application/json` | identical |  |
| 10 | `airports/{iata}` | airport LIS | `airports/LIS` | 200 | 200 | `application/json` | identical |  |
| 11 | `airports/{iata}` | airport FCO | `airports/FCO` | 200 | 200 | `application/json` | identical |  |
| 12 | `airports/{iata}` | airport MUC | `airports/MUC` | 200 | 200 | `application/json` | identical |  |
| 13 | `airports/{iata}` | airport BCN | `airports/BCN` | 200 | 200 | `application/json` | identical |  |
| 14 | `airports/{iata}` | airport DXB | `airports/DXB` | 200 | 200 | `application/json` | identical |  |
| 15 | `airports/{iata}` | airport BOM | `airports/BOM` | 200 | 200 | `application/json` | identical |  |
| 16 | `airports/{iata}` | airport lowercase lhr | `airports/lhr` | 200 | 200 | `application/json` | identical |  |
| 17 | `airports/{iata}` | airport mixed case Nce | `airports/Nce` | 200 | 200 | `application/json` | identical |  |
| 18 | `airports/{iata}` | airport padded nce | `airports/%20nce%20` | 200 | 200 | `application/json` | identical |  |
| 19 | `airports/{iata}` | airport missing ZZZ | `airports/ZZZ` | 404 | 404 | `application/problem+json` | identical |  |
| 20 | `airports/{iata}` | airport too long NCEX | `airports/NCEX` | 404 | 404 | `application/problem+json` | identical |  |
| 21 | `airports/{iata}` | airport numeric 123 | `airports/123` | 404 | 404 | `application/problem+json` | identical |  |
| 22 | `airports/{iata}` | airport trailing slash | `airports/` | 404 | 404 | `application/problem+json` | identical |  |
| 23 | `airports/{iata}` | airport extra segment | `airports/NCE/x` | 404 | 404 | `application/problem+json` | identical |  |
| 24 | `airlines` | airlines list | `airlines` | 200 | 200 | `application/json` | identical |  |
| 25 | `airlines/{iata}` | airline AF | `airlines/AF` | 200 | 200 | `application/json` | identical |  |
| 26 | `airlines/{iata}` | airline BA | `airlines/BA` | 200 | 200 | `application/json` | identical |  |
| 27 | `airlines/{iata}` | airline DL | `airlines/DL` | 200 | 200 | `application/json` | identical |  |
| 28 | `airlines/{iata}` | airline SQ | `airlines/SQ` | 200 | 200 | `application/json` | identical |  |
| 29 | `airlines/{iata}` | airline IB | `airlines/IB` | 200 | 200 | `application/json` | identical |  |
| 30 | `airlines/{iata}` | airline LH | `airlines/LH` | 200 | 200 | `application/json` | identical |  |
| 31 | `airlines/{iata}` | airline KL | `airlines/KL` | 200 | 200 | `application/json` | identical |  |
| 32 | `airlines/{iata}` | airline TP | `airlines/TP` | 200 | 200 | `application/json` | identical |  |
| 33 | `airlines/{iata}` | airline AZ | `airlines/AZ` | 200 | 200 | `application/json` | identical |  |
| 34 | `airlines/{iata}` | airline EK | `airlines/EK` | 200 | 200 | `application/json` | identical |  |
| 35 | `airlines/{iata}` | airline AI | `airlines/AI` | 200 | 200 | `application/json` | identical |  |
| 36 | `airlines/{iata}` | airline lowercase ek | `airlines/ek` | 200 | 200 | `application/json` | identical |  |
| 37 | `airlines/{iata}` | airline padded ba | `airlines/%20ba` | 200 | 200 | `application/json` | identical |  |
| 38 | `airlines/{iata}` | airline missing XX | `airlines/XX` | 404 | 404 | `application/problem+json` | identical |  |
| 39 | `airlines/{iata}` | airline too long AFX | `airlines/AFX` | 404 | 404 | `application/problem+json` | identical |  |
| 40 | `flights` | flights NCE-LHR d+0 | `flights?origin=NCE&destination=LHR&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 41 | `flights` | flights NCE-LHR d+7 | `flights?origin=NCE&destination=LHR&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 42 | `flights` | flights NCE-LHR d+15 | `flights?origin=NCE&destination=LHR&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 43 | `flights` | flights NCE-LHR d+29 | `flights?origin=NCE&destination=LHR&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 44 | `flights` | flights LHR-NCE d+0 | `flights?origin=LHR&destination=NCE&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 45 | `flights` | flights LHR-NCE d+7 | `flights?origin=LHR&destination=NCE&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 46 | `flights` | flights LHR-NCE d+15 | `flights?origin=LHR&destination=NCE&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 47 | `flights` | flights LHR-NCE d+29 | `flights?origin=LHR&destination=NCE&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 48 | `flights` | flights NCE-CDG d+0 | `flights?origin=NCE&destination=CDG&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 49 | `flights` | flights NCE-CDG d+7 | `flights?origin=NCE&destination=CDG&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 50 | `flights` | flights NCE-CDG d+15 | `flights?origin=NCE&destination=CDG&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 51 | `flights` | flights NCE-CDG d+29 | `flights?origin=NCE&destination=CDG&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 52 | `flights` | flights CDG-NCE d+0 | `flights?origin=CDG&destination=NCE&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 53 | `flights` | flights CDG-NCE d+7 | `flights?origin=CDG&destination=NCE&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 54 | `flights` | flights CDG-NCE d+15 | `flights?origin=CDG&destination=NCE&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 55 | `flights` | flights CDG-NCE d+29 | `flights?origin=CDG&destination=NCE&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 56 | `flights` | flights CDG-JFK d+0 | `flights?origin=CDG&destination=JFK&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 57 | `flights` | flights CDG-JFK d+7 | `flights?origin=CDG&destination=JFK&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 58 | `flights` | flights CDG-JFK d+15 | `flights?origin=CDG&destination=JFK&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 59 | `flights` | flights CDG-JFK d+29 | `flights?origin=CDG&destination=JFK&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 60 | `flights` | flights JFK-CDG d+0 | `flights?origin=JFK&destination=CDG&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 61 | `flights` | flights JFK-CDG d+7 | `flights?origin=JFK&destination=CDG&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 62 | `flights` | flights JFK-CDG d+15 | `flights?origin=JFK&destination=CDG&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 63 | `flights` | flights JFK-CDG d+29 | `flights?origin=JFK&destination=CDG&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 64 | `flights` | flights LHR-SIN d+0 | `flights?origin=LHR&destination=SIN&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 65 | `flights` | flights LHR-SIN d+7 | `flights?origin=LHR&destination=SIN&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 66 | `flights` | flights LHR-SIN d+15 | `flights?origin=LHR&destination=SIN&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 67 | `flights` | flights LHR-SIN d+29 | `flights?origin=LHR&destination=SIN&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 68 | `flights` | flights SIN-LHR d+0 | `flights?origin=SIN&destination=LHR&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 69 | `flights` | flights SIN-LHR d+7 | `flights?origin=SIN&destination=LHR&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 70 | `flights` | flights SIN-LHR d+15 | `flights?origin=SIN&destination=LHR&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 71 | `flights` | flights SIN-LHR d+29 | `flights?origin=SIN&destination=LHR&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 72 | `flights` | flights MAD-FRA d+0 | `flights?origin=MAD&destination=FRA&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 73 | `flights` | flights MAD-FRA d+7 | `flights?origin=MAD&destination=FRA&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 74 | `flights` | flights MAD-FRA d+15 | `flights?origin=MAD&destination=FRA&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 75 | `flights` | flights MAD-FRA d+29 | `flights?origin=MAD&destination=FRA&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 76 | `flights` | flights FRA-MAD d+0 | `flights?origin=FRA&destination=MAD&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 77 | `flights` | flights FRA-MAD d+7 | `flights?origin=FRA&destination=MAD&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 78 | `flights` | flights FRA-MAD d+15 | `flights?origin=FRA&destination=MAD&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 79 | `flights` | flights FRA-MAD d+29 | `flights?origin=FRA&destination=MAD&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 80 | `flights` | flights AMS-LIS d+0 | `flights?origin=AMS&destination=LIS&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 81 | `flights` | flights AMS-LIS d+7 | `flights?origin=AMS&destination=LIS&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 82 | `flights` | flights AMS-LIS d+15 | `flights?origin=AMS&destination=LIS&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 83 | `flights` | flights AMS-LIS d+29 | `flights?origin=AMS&destination=LIS&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 84 | `flights` | flights LIS-AMS d+0 | `flights?origin=LIS&destination=AMS&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 85 | `flights` | flights LIS-AMS d+7 | `flights?origin=LIS&destination=AMS&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 86 | `flights` | flights LIS-AMS d+15 | `flights?origin=LIS&destination=AMS&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 87 | `flights` | flights LIS-AMS d+29 | `flights?origin=LIS&destination=AMS&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 88 | `flights` | flights FCO-MUC d+0 | `flights?origin=FCO&destination=MUC&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 89 | `flights` | flights FCO-MUC d+7 | `flights?origin=FCO&destination=MUC&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 90 | `flights` | flights FCO-MUC d+15 | `flights?origin=FCO&destination=MUC&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 91 | `flights` | flights FCO-MUC d+29 | `flights?origin=FCO&destination=MUC&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 92 | `flights` | flights MUC-FCO d+0 | `flights?origin=MUC&destination=FCO&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 93 | `flights` | flights MUC-FCO d+7 | `flights?origin=MUC&destination=FCO&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 94 | `flights` | flights MUC-FCO d+15 | `flights?origin=MUC&destination=FCO&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 95 | `flights` | flights MUC-FCO d+29 | `flights?origin=MUC&destination=FCO&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 96 | `flights` | flights BCN-DXB d+0 | `flights?origin=BCN&destination=DXB&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 97 | `flights` | flights BCN-DXB d+7 | `flights?origin=BCN&destination=DXB&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 98 | `flights` | flights BCN-DXB d+15 | `flights?origin=BCN&destination=DXB&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 99 | `flights` | flights BCN-DXB d+29 | `flights?origin=BCN&destination=DXB&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 100 | `flights` | flights DXB-BCN d+0 | `flights?origin=DXB&destination=BCN&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 101 | `flights` | flights DXB-BCN d+7 | `flights?origin=DXB&destination=BCN&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 102 | `flights` | flights DXB-BCN d+15 | `flights?origin=DXB&destination=BCN&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 103 | `flights` | flights DXB-BCN d+29 | `flights?origin=DXB&destination=BCN&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 104 | `flights` | flights DXB-BOM d+0 | `flights?origin=DXB&destination=BOM&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 105 | `flights` | flights DXB-BOM d+7 | `flights?origin=DXB&destination=BOM&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 106 | `flights` | flights DXB-BOM d+15 | `flights?origin=DXB&destination=BOM&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 107 | `flights` | flights DXB-BOM d+29 | `flights?origin=DXB&destination=BOM&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 108 | `flights` | flights BOM-DXB d+0 | `flights?origin=BOM&destination=DXB&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 109 | `flights` | flights BOM-DXB d+7 | `flights?origin=BOM&destination=DXB&date=2026-11-12` | 200 | 200 | `application/json` | identical |  |
| 110 | `flights` | flights BOM-DXB d+15 | `flights?origin=BOM&destination=DXB&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 111 | `flights` | flights BOM-DXB d+29 | `flights?origin=BOM&destination=DXB&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 112 | `flights` | flights NCE-LHR d+30 outside window | `flights?origin=NCE&destination=LHR&date=2026-12-05` | 200 | 200 | `application/json` | identical |  |
| 113 | `flights` | flights NCE-LHR d-1 before window | `flights?origin=NCE&destination=LHR&date=2026-11-04` | 200 | 200 | `application/json` | identical |  |
| 114 | `flights` | flights lowercase iata | `flights?origin=nce&destination=lhr&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 115 | `flights` | flights padded iata | `flights?origin=%20NCE&destination=LHR%20&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 116 | `flights` | flights unserved route NCE-JFK | `flights?origin=NCE&destination=JFK&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 117 | `flights` | flights unknown origin | `flights?origin=QQQ&destination=LHR&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 118 | `flights` | flights unknown destination | `flights?origin=NCE&destination=QQQ&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 119 | `flights` | flights same origin and destination | `flights?origin=NCE&destination=nce&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 120 | `flights` | flights repeated origin | `flights?origin=NCE&origin=CDG&destination=LHR&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 121 | `flights` | flights missing origin | `flights?destination=LHR&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 122 | `flights` | flights missing destination | `flights?origin=NCE&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 123 | `flights` | flights missing date | `flights?origin=NCE&destination=LHR` | 400 | 400 | `application/problem+json` | identical |  |
| 124 | `flights` | flights no params | `flights` | 400 | 400 | `application/problem+json` | identical |  |
| 125 | `flights` | flights malformed date word | `flights?origin=NCE&destination=LHR&date=bad` | 400 | 400 | `application/problem+json` | identical |  |
| 126 | `flights` | flights malformed date feb 30 | `flights?origin=NCE&destination=LHR&date=2026-02-30` | 400 | 400 | `application/problem+json` | identical |  |
| 127 | `flights` | flights malformed date short day | `flights?origin=NCE&destination=LHR&date=2026-11-4` | 400 | 400 | `application/problem+json` | identical |  |
| 128 | `flights` | flights malformed date compact | `flights?origin=NCE&destination=LHR&date=20261105` | 400 | 400 | `application/problem+json` | identical |  |
| 129 | `flights` | flights malformed date empty | `flights?origin=NCE&destination=LHR&date=` | 400 | 400 | `application/problem+json` | identical |  |
| 130 | `flights/{id}` | flight first | `flights/1` | 200 | 200 | `application/json` | identical |  |
| 131 | `flights/{id}` | flight second | `flights/2` | 200 | 200 | `application/json` | identical |  |
| 132 | `flights/{id}` | flight middle | `flights/1560` | 200 | 200 | `application/json` | identical |  |
| 133 | `flights/{id}` | flight last | `flights/3120` | 200 | 200 | `application/json` | identical |  |
| 134 | `flights/{id}` | flight missing after last | `flights/3121` | 404 | 404 | `application/problem+json` | identical |  |
| 135 | `flights/{id}` | flight zero | `flights/0` | 404 | 404 | `application/problem+json` | identical |  |
| 136 | `flights/{id}` | flight negative | `flights/-1` | 404 | 404 | `application/problem+json` | identical |  |
| 137 | `flights/{id}` | flight non numeric | `flights/abc` | 400 | 400 | `application/problem+json` | identical |  |
| 138 | `flights/{id}` | flight decimal | `flights/1.5` | 400 | 400 | `application/problem+json` | identical |  |
| 139 | `flights/{id}` | flight overflow | `flights/99999999999999999999` | 400 | 400 | `application/problem+json` | identical |  |
| 140 | `offers` | offers NCE-LHR d+0 adults=1 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 141 | `offers` | offers NCE-LHR d+0 adults=1 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 142 | `offers` | offers NCE-LHR d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 143 | `offers` | offers NCE-LHR d+0 adults=1 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 144 | `offers` | offers NCE-LHR d+0 adults=1 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 145 | `offers` | offers NCE-LHR d+0 adults=2 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 146 | `offers` | offers NCE-LHR d+0 adults=2 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 147 | `offers` | offers NCE-LHR d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 148 | `offers` | offers NCE-LHR d+0 adults=2 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 149 | `offers` | offers NCE-LHR d+0 adults=2 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 150 | `offers` | offers NCE-LHR d+0 adults=5 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 151 | `offers` | offers NCE-LHR d+0 adults=5 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 152 | `offers` | offers NCE-LHR d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 153 | `offers` | offers NCE-LHR d+0 adults=5 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 154 | `offers` | offers NCE-LHR d+0 adults=5 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 155 | `offers` | offers NCE-LHR d+0 adults=9 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 156 | `offers` | offers NCE-LHR d+0 adults=9 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 157 | `offers` | offers NCE-LHR d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 158 | `offers` | offers NCE-LHR d+0 adults=9 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 159 | `offers` | offers NCE-LHR d+0 adults=9 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 160 | `offers` | offers NCE-LHR d+7 adults=1 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 161 | `offers` | offers NCE-LHR d+7 adults=1 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 162 | `offers` | offers NCE-LHR d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 163 | `offers` | offers NCE-LHR d+7 adults=1 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 164 | `offers` | offers NCE-LHR d+7 adults=1 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 165 | `offers` | offers NCE-LHR d+7 adults=2 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 166 | `offers` | offers NCE-LHR d+7 adults=2 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 167 | `offers` | offers NCE-LHR d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 168 | `offers` | offers NCE-LHR d+7 adults=2 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 169 | `offers` | offers NCE-LHR d+7 adults=2 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 170 | `offers` | offers NCE-LHR d+7 adults=5 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 171 | `offers` | offers NCE-LHR d+7 adults=5 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 172 | `offers` | offers NCE-LHR d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 173 | `offers` | offers NCE-LHR d+7 adults=5 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 174 | `offers` | offers NCE-LHR d+7 adults=5 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 175 | `offers` | offers NCE-LHR d+7 adults=9 cabin=any | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 176 | `offers` | offers NCE-LHR d+7 adults=9 cabin=ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 177 | `offers` | offers NCE-LHR d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 178 | `offers` | offers NCE-LHR d+7 adults=9 cabin=BUSINESS | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 179 | `offers` | offers NCE-LHR d+7 adults=9 cabin=FIRST | `offers?origin=NCE&destination=LHR&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 180 | `offers` | offers NCE-LHR d+15 defaults | `offers?origin=NCE&destination=LHR&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 181 | `offers` | offers NCE-LHR d+29 defaults | `offers?origin=NCE&destination=LHR&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 182 | `offers` | offers LHR-NCE d+0 defaults | `offers?origin=LHR&destination=NCE&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 183 | `offers` | offers NCE-CDG d+0 adults=1 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 184 | `offers` | offers NCE-CDG d+0 adults=1 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 185 | `offers` | offers NCE-CDG d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 186 | `offers` | offers NCE-CDG d+0 adults=1 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 187 | `offers` | offers NCE-CDG d+0 adults=1 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 188 | `offers` | offers NCE-CDG d+0 adults=2 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 189 | `offers` | offers NCE-CDG d+0 adults=2 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 190 | `offers` | offers NCE-CDG d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 191 | `offers` | offers NCE-CDG d+0 adults=2 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 192 | `offers` | offers NCE-CDG d+0 adults=2 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 193 | `offers` | offers NCE-CDG d+0 adults=5 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 194 | `offers` | offers NCE-CDG d+0 adults=5 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 195 | `offers` | offers NCE-CDG d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 196 | `offers` | offers NCE-CDG d+0 adults=5 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 197 | `offers` | offers NCE-CDG d+0 adults=5 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 198 | `offers` | offers NCE-CDG d+0 adults=9 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 199 | `offers` | offers NCE-CDG d+0 adults=9 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 200 | `offers` | offers NCE-CDG d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 201 | `offers` | offers NCE-CDG d+0 adults=9 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 202 | `offers` | offers NCE-CDG d+0 adults=9 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 203 | `offers` | offers NCE-CDG d+7 adults=1 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 204 | `offers` | offers NCE-CDG d+7 adults=1 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 205 | `offers` | offers NCE-CDG d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 206 | `offers` | offers NCE-CDG d+7 adults=1 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 207 | `offers` | offers NCE-CDG d+7 adults=1 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 208 | `offers` | offers NCE-CDG d+7 adults=2 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 209 | `offers` | offers NCE-CDG d+7 adults=2 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 210 | `offers` | offers NCE-CDG d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 211 | `offers` | offers NCE-CDG d+7 adults=2 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 212 | `offers` | offers NCE-CDG d+7 adults=2 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 213 | `offers` | offers NCE-CDG d+7 adults=5 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 214 | `offers` | offers NCE-CDG d+7 adults=5 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 215 | `offers` | offers NCE-CDG d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 216 | `offers` | offers NCE-CDG d+7 adults=5 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 217 | `offers` | offers NCE-CDG d+7 adults=5 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 218 | `offers` | offers NCE-CDG d+7 adults=9 cabin=any | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 219 | `offers` | offers NCE-CDG d+7 adults=9 cabin=ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 220 | `offers` | offers NCE-CDG d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 221 | `offers` | offers NCE-CDG d+7 adults=9 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 222 | `offers` | offers NCE-CDG d+7 adults=9 cabin=FIRST | `offers?origin=NCE&destination=CDG&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 223 | `offers` | offers NCE-CDG d+15 defaults | `offers?origin=NCE&destination=CDG&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 224 | `offers` | offers NCE-CDG d+29 defaults | `offers?origin=NCE&destination=CDG&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 225 | `offers` | offers CDG-NCE d+0 defaults | `offers?origin=CDG&destination=NCE&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 226 | `offers` | offers CDG-JFK d+0 adults=1 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 227 | `offers` | offers CDG-JFK d+0 adults=1 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 228 | `offers` | offers CDG-JFK d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 229 | `offers` | offers CDG-JFK d+0 adults=1 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 230 | `offers` | offers CDG-JFK d+0 adults=1 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 231 | `offers` | offers CDG-JFK d+0 adults=2 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 232 | `offers` | offers CDG-JFK d+0 adults=2 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 233 | `offers` | offers CDG-JFK d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 234 | `offers` | offers CDG-JFK d+0 adults=2 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 235 | `offers` | offers CDG-JFK d+0 adults=2 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 236 | `offers` | offers CDG-JFK d+0 adults=5 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 237 | `offers` | offers CDG-JFK d+0 adults=5 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 238 | `offers` | offers CDG-JFK d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 239 | `offers` | offers CDG-JFK d+0 adults=5 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 240 | `offers` | offers CDG-JFK d+0 adults=5 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 241 | `offers` | offers CDG-JFK d+0 adults=9 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 242 | `offers` | offers CDG-JFK d+0 adults=9 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 243 | `offers` | offers CDG-JFK d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 244 | `offers` | offers CDG-JFK d+0 adults=9 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 245 | `offers` | offers CDG-JFK d+0 adults=9 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 246 | `offers` | offers CDG-JFK d+7 adults=1 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 247 | `offers` | offers CDG-JFK d+7 adults=1 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 248 | `offers` | offers CDG-JFK d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 249 | `offers` | offers CDG-JFK d+7 adults=1 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 250 | `offers` | offers CDG-JFK d+7 adults=1 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 251 | `offers` | offers CDG-JFK d+7 adults=2 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 252 | `offers` | offers CDG-JFK d+7 adults=2 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 253 | `offers` | offers CDG-JFK d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 254 | `offers` | offers CDG-JFK d+7 adults=2 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 255 | `offers` | offers CDG-JFK d+7 adults=2 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 256 | `offers` | offers CDG-JFK d+7 adults=5 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 257 | `offers` | offers CDG-JFK d+7 adults=5 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 258 | `offers` | offers CDG-JFK d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 259 | `offers` | offers CDG-JFK d+7 adults=5 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 260 | `offers` | offers CDG-JFK d+7 adults=5 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 261 | `offers` | offers CDG-JFK d+7 adults=9 cabin=any | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 262 | `offers` | offers CDG-JFK d+7 adults=9 cabin=ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 263 | `offers` | offers CDG-JFK d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 264 | `offers` | offers CDG-JFK d+7 adults=9 cabin=BUSINESS | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 265 | `offers` | offers CDG-JFK d+7 adults=9 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 266 | `offers` | offers CDG-JFK d+15 defaults | `offers?origin=CDG&destination=JFK&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 267 | `offers` | offers CDG-JFK d+29 defaults | `offers?origin=CDG&destination=JFK&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 268 | `offers` | offers JFK-CDG d+0 defaults | `offers?origin=JFK&destination=CDG&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 269 | `offers` | offers LHR-SIN d+0 adults=1 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 270 | `offers` | offers LHR-SIN d+0 adults=1 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 271 | `offers` | offers LHR-SIN d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 272 | `offers` | offers LHR-SIN d+0 adults=1 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 273 | `offers` | offers LHR-SIN d+0 adults=1 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 274 | `offers` | offers LHR-SIN d+0 adults=2 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 275 | `offers` | offers LHR-SIN d+0 adults=2 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 276 | `offers` | offers LHR-SIN d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 277 | `offers` | offers LHR-SIN d+0 adults=2 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 278 | `offers` | offers LHR-SIN d+0 adults=2 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 279 | `offers` | offers LHR-SIN d+0 adults=5 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 280 | `offers` | offers LHR-SIN d+0 adults=5 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 281 | `offers` | offers LHR-SIN d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 282 | `offers` | offers LHR-SIN d+0 adults=5 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 283 | `offers` | offers LHR-SIN d+0 adults=5 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 284 | `offers` | offers LHR-SIN d+0 adults=9 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 285 | `offers` | offers LHR-SIN d+0 adults=9 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 286 | `offers` | offers LHR-SIN d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 287 | `offers` | offers LHR-SIN d+0 adults=9 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 288 | `offers` | offers LHR-SIN d+0 adults=9 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 289 | `offers` | offers LHR-SIN d+7 adults=1 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 290 | `offers` | offers LHR-SIN d+7 adults=1 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 291 | `offers` | offers LHR-SIN d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 292 | `offers` | offers LHR-SIN d+7 adults=1 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 293 | `offers` | offers LHR-SIN d+7 adults=1 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 294 | `offers` | offers LHR-SIN d+7 adults=2 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 295 | `offers` | offers LHR-SIN d+7 adults=2 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 296 | `offers` | offers LHR-SIN d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 297 | `offers` | offers LHR-SIN d+7 adults=2 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 298 | `offers` | offers LHR-SIN d+7 adults=2 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 299 | `offers` | offers LHR-SIN d+7 adults=5 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 300 | `offers` | offers LHR-SIN d+7 adults=5 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 301 | `offers` | offers LHR-SIN d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 302 | `offers` | offers LHR-SIN d+7 adults=5 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 303 | `offers` | offers LHR-SIN d+7 adults=5 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 304 | `offers` | offers LHR-SIN d+7 adults=9 cabin=any | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 305 | `offers` | offers LHR-SIN d+7 adults=9 cabin=ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 306 | `offers` | offers LHR-SIN d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 307 | `offers` | offers LHR-SIN d+7 adults=9 cabin=BUSINESS | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 308 | `offers` | offers LHR-SIN d+7 adults=9 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 309 | `offers` | offers LHR-SIN d+15 defaults | `offers?origin=LHR&destination=SIN&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 310 | `offers` | offers LHR-SIN d+29 defaults | `offers?origin=LHR&destination=SIN&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 311 | `offers` | offers SIN-LHR d+0 defaults | `offers?origin=SIN&destination=LHR&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 312 | `offers` | offers MAD-FRA d+0 adults=1 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 313 | `offers` | offers MAD-FRA d+0 adults=1 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 314 | `offers` | offers MAD-FRA d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 315 | `offers` | offers MAD-FRA d+0 adults=1 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 316 | `offers` | offers MAD-FRA d+0 adults=1 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 317 | `offers` | offers MAD-FRA d+0 adults=2 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 318 | `offers` | offers MAD-FRA d+0 adults=2 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 319 | `offers` | offers MAD-FRA d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 320 | `offers` | offers MAD-FRA d+0 adults=2 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 321 | `offers` | offers MAD-FRA d+0 adults=2 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 322 | `offers` | offers MAD-FRA d+0 adults=5 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 323 | `offers` | offers MAD-FRA d+0 adults=5 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 324 | `offers` | offers MAD-FRA d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 325 | `offers` | offers MAD-FRA d+0 adults=5 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 326 | `offers` | offers MAD-FRA d+0 adults=5 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 327 | `offers` | offers MAD-FRA d+0 adults=9 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 328 | `offers` | offers MAD-FRA d+0 adults=9 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 329 | `offers` | offers MAD-FRA d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 330 | `offers` | offers MAD-FRA d+0 adults=9 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 331 | `offers` | offers MAD-FRA d+0 adults=9 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 332 | `offers` | offers MAD-FRA d+7 adults=1 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 333 | `offers` | offers MAD-FRA d+7 adults=1 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 334 | `offers` | offers MAD-FRA d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 335 | `offers` | offers MAD-FRA d+7 adults=1 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 336 | `offers` | offers MAD-FRA d+7 adults=1 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 337 | `offers` | offers MAD-FRA d+7 adults=2 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 338 | `offers` | offers MAD-FRA d+7 adults=2 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 339 | `offers` | offers MAD-FRA d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 340 | `offers` | offers MAD-FRA d+7 adults=2 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 341 | `offers` | offers MAD-FRA d+7 adults=2 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 342 | `offers` | offers MAD-FRA d+7 adults=5 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 343 | `offers` | offers MAD-FRA d+7 adults=5 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 344 | `offers` | offers MAD-FRA d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 345 | `offers` | offers MAD-FRA d+7 adults=5 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 346 | `offers` | offers MAD-FRA d+7 adults=5 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 347 | `offers` | offers MAD-FRA d+7 adults=9 cabin=any | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 348 | `offers` | offers MAD-FRA d+7 adults=9 cabin=ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 349 | `offers` | offers MAD-FRA d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 350 | `offers` | offers MAD-FRA d+7 adults=9 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 351 | `offers` | offers MAD-FRA d+7 adults=9 cabin=FIRST | `offers?origin=MAD&destination=FRA&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 352 | `offers` | offers MAD-FRA d+15 defaults | `offers?origin=MAD&destination=FRA&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 353 | `offers` | offers MAD-FRA d+29 defaults | `offers?origin=MAD&destination=FRA&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 354 | `offers` | offers FRA-MAD d+0 defaults | `offers?origin=FRA&destination=MAD&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 355 | `offers` | offers AMS-LIS d+0 adults=1 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 356 | `offers` | offers AMS-LIS d+0 adults=1 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 357 | `offers` | offers AMS-LIS d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 358 | `offers` | offers AMS-LIS d+0 adults=1 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 359 | `offers` | offers AMS-LIS d+0 adults=1 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 360 | `offers` | offers AMS-LIS d+0 adults=2 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 361 | `offers` | offers AMS-LIS d+0 adults=2 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 362 | `offers` | offers AMS-LIS d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 363 | `offers` | offers AMS-LIS d+0 adults=2 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 364 | `offers` | offers AMS-LIS d+0 adults=2 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 365 | `offers` | offers AMS-LIS d+0 adults=5 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 366 | `offers` | offers AMS-LIS d+0 adults=5 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 367 | `offers` | offers AMS-LIS d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 368 | `offers` | offers AMS-LIS d+0 adults=5 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 369 | `offers` | offers AMS-LIS d+0 adults=5 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 370 | `offers` | offers AMS-LIS d+0 adults=9 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 371 | `offers` | offers AMS-LIS d+0 adults=9 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 372 | `offers` | offers AMS-LIS d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 373 | `offers` | offers AMS-LIS d+0 adults=9 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 374 | `offers` | offers AMS-LIS d+0 adults=9 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 375 | `offers` | offers AMS-LIS d+7 adults=1 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 376 | `offers` | offers AMS-LIS d+7 adults=1 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 377 | `offers` | offers AMS-LIS d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 378 | `offers` | offers AMS-LIS d+7 adults=1 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 379 | `offers` | offers AMS-LIS d+7 adults=1 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 380 | `offers` | offers AMS-LIS d+7 adults=2 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 381 | `offers` | offers AMS-LIS d+7 adults=2 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 382 | `offers` | offers AMS-LIS d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 383 | `offers` | offers AMS-LIS d+7 adults=2 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 384 | `offers` | offers AMS-LIS d+7 adults=2 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 385 | `offers` | offers AMS-LIS d+7 adults=5 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 386 | `offers` | offers AMS-LIS d+7 adults=5 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 387 | `offers` | offers AMS-LIS d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 388 | `offers` | offers AMS-LIS d+7 adults=5 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 389 | `offers` | offers AMS-LIS d+7 adults=5 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 390 | `offers` | offers AMS-LIS d+7 adults=9 cabin=any | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 391 | `offers` | offers AMS-LIS d+7 adults=9 cabin=ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 392 | `offers` | offers AMS-LIS d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 393 | `offers` | offers AMS-LIS d+7 adults=9 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 394 | `offers` | offers AMS-LIS d+7 adults=9 cabin=FIRST | `offers?origin=AMS&destination=LIS&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 395 | `offers` | offers AMS-LIS d+15 defaults | `offers?origin=AMS&destination=LIS&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 396 | `offers` | offers AMS-LIS d+29 defaults | `offers?origin=AMS&destination=LIS&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 397 | `offers` | offers LIS-AMS d+0 defaults | `offers?origin=LIS&destination=AMS&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 398 | `offers` | offers FCO-MUC d+0 adults=1 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 399 | `offers` | offers FCO-MUC d+0 adults=1 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 400 | `offers` | offers FCO-MUC d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 401 | `offers` | offers FCO-MUC d+0 adults=1 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 402 | `offers` | offers FCO-MUC d+0 adults=1 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 403 | `offers` | offers FCO-MUC d+0 adults=2 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 404 | `offers` | offers FCO-MUC d+0 adults=2 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 405 | `offers` | offers FCO-MUC d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 406 | `offers` | offers FCO-MUC d+0 adults=2 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 407 | `offers` | offers FCO-MUC d+0 adults=2 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 408 | `offers` | offers FCO-MUC d+0 adults=5 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 409 | `offers` | offers FCO-MUC d+0 adults=5 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 410 | `offers` | offers FCO-MUC d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 411 | `offers` | offers FCO-MUC d+0 adults=5 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 412 | `offers` | offers FCO-MUC d+0 adults=5 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 413 | `offers` | offers FCO-MUC d+0 adults=9 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 414 | `offers` | offers FCO-MUC d+0 adults=9 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 415 | `offers` | offers FCO-MUC d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 416 | `offers` | offers FCO-MUC d+0 adults=9 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 417 | `offers` | offers FCO-MUC d+0 adults=9 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 418 | `offers` | offers FCO-MUC d+7 adults=1 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 419 | `offers` | offers FCO-MUC d+7 adults=1 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 420 | `offers` | offers FCO-MUC d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 421 | `offers` | offers FCO-MUC d+7 adults=1 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 422 | `offers` | offers FCO-MUC d+7 adults=1 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 423 | `offers` | offers FCO-MUC d+7 adults=2 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 424 | `offers` | offers FCO-MUC d+7 adults=2 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 425 | `offers` | offers FCO-MUC d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 426 | `offers` | offers FCO-MUC d+7 adults=2 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 427 | `offers` | offers FCO-MUC d+7 adults=2 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 428 | `offers` | offers FCO-MUC d+7 adults=5 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 429 | `offers` | offers FCO-MUC d+7 adults=5 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 430 | `offers` | offers FCO-MUC d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 431 | `offers` | offers FCO-MUC d+7 adults=5 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 432 | `offers` | offers FCO-MUC d+7 adults=5 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 433 | `offers` | offers FCO-MUC d+7 adults=9 cabin=any | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 434 | `offers` | offers FCO-MUC d+7 adults=9 cabin=ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 435 | `offers` | offers FCO-MUC d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 436 | `offers` | offers FCO-MUC d+7 adults=9 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 437 | `offers` | offers FCO-MUC d+7 adults=9 cabin=FIRST | `offers?origin=FCO&destination=MUC&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 438 | `offers` | offers FCO-MUC d+15 defaults | `offers?origin=FCO&destination=MUC&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 439 | `offers` | offers FCO-MUC d+29 defaults | `offers?origin=FCO&destination=MUC&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 440 | `offers` | offers MUC-FCO d+0 defaults | `offers?origin=MUC&destination=FCO&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 441 | `offers` | offers BCN-DXB d+0 adults=1 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 442 | `offers` | offers BCN-DXB d+0 adults=1 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 443 | `offers` | offers BCN-DXB d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 444 | `offers` | offers BCN-DXB d+0 adults=1 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 445 | `offers` | offers BCN-DXB d+0 adults=1 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 446 | `offers` | offers BCN-DXB d+0 adults=2 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 447 | `offers` | offers BCN-DXB d+0 adults=2 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 448 | `offers` | offers BCN-DXB d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 449 | `offers` | offers BCN-DXB d+0 adults=2 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 450 | `offers` | offers BCN-DXB d+0 adults=2 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 451 | `offers` | offers BCN-DXB d+0 adults=5 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 452 | `offers` | offers BCN-DXB d+0 adults=5 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 453 | `offers` | offers BCN-DXB d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 454 | `offers` | offers BCN-DXB d+0 adults=5 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 455 | `offers` | offers BCN-DXB d+0 adults=5 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 456 | `offers` | offers BCN-DXB d+0 adults=9 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 457 | `offers` | offers BCN-DXB d+0 adults=9 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 458 | `offers` | offers BCN-DXB d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 459 | `offers` | offers BCN-DXB d+0 adults=9 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 460 | `offers` | offers BCN-DXB d+0 adults=9 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 461 | `offers` | offers BCN-DXB d+7 adults=1 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 462 | `offers` | offers BCN-DXB d+7 adults=1 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 463 | `offers` | offers BCN-DXB d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 464 | `offers` | offers BCN-DXB d+7 adults=1 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 465 | `offers` | offers BCN-DXB d+7 adults=1 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 466 | `offers` | offers BCN-DXB d+7 adults=2 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 467 | `offers` | offers BCN-DXB d+7 adults=2 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 468 | `offers` | offers BCN-DXB d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 469 | `offers` | offers BCN-DXB d+7 adults=2 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 470 | `offers` | offers BCN-DXB d+7 adults=2 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 471 | `offers` | offers BCN-DXB d+7 adults=5 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 472 | `offers` | offers BCN-DXB d+7 adults=5 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 473 | `offers` | offers BCN-DXB d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 474 | `offers` | offers BCN-DXB d+7 adults=5 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 475 | `offers` | offers BCN-DXB d+7 adults=5 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 476 | `offers` | offers BCN-DXB d+7 adults=9 cabin=any | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 477 | `offers` | offers BCN-DXB d+7 adults=9 cabin=ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 478 | `offers` | offers BCN-DXB d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 479 | `offers` | offers BCN-DXB d+7 adults=9 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 480 | `offers` | offers BCN-DXB d+7 adults=9 cabin=FIRST | `offers?origin=BCN&destination=DXB&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 481 | `offers` | offers BCN-DXB d+15 defaults | `offers?origin=BCN&destination=DXB&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 482 | `offers` | offers BCN-DXB d+29 defaults | `offers?origin=BCN&destination=DXB&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 483 | `offers` | offers DXB-BCN d+0 defaults | `offers?origin=DXB&destination=BCN&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 484 | `offers` | offers DXB-BOM d+0 adults=1 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=1` | 200 | 200 | `application/json` | identical |  |
| 485 | `offers` | offers DXB-BOM d+0 adults=1 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 486 | `offers` | offers DXB-BOM d+0 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 487 | `offers` | offers DXB-BOM d+0 adults=1 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 488 | `offers` | offers DXB-BOM d+0 adults=1 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 489 | `offers` | offers DXB-BOM d+0 adults=2 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=2` | 200 | 200 | `application/json` | identical |  |
| 490 | `offers` | offers DXB-BOM d+0 adults=2 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 491 | `offers` | offers DXB-BOM d+0 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 492 | `offers` | offers DXB-BOM d+0 adults=2 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 493 | `offers` | offers DXB-BOM d+0 adults=2 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 494 | `offers` | offers DXB-BOM d+0 adults=5 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=5` | 200 | 200 | `application/json` | identical |  |
| 495 | `offers` | offers DXB-BOM d+0 adults=5 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 496 | `offers` | offers DXB-BOM d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 497 | `offers` | offers DXB-BOM d+0 adults=5 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 498 | `offers` | offers DXB-BOM d+0 adults=5 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 499 | `offers` | offers DXB-BOM d+0 adults=9 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=9` | 200 | 200 | `application/json` | identical |  |
| 500 | `offers` | offers DXB-BOM d+0 adults=9 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 501 | `offers` | offers DXB-BOM d+0 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 502 | `offers` | offers DXB-BOM d+0 adults=9 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 503 | `offers` | offers DXB-BOM d+0 adults=9 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 504 | `offers` | offers DXB-BOM d+7 adults=1 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=1` | 200 | 200 | `application/json` | identical |  |
| 505 | `offers` | offers DXB-BOM d+7 adults=1 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=1&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 506 | `offers` | offers DXB-BOM d+7 adults=1 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=1&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 507 | `offers` | offers DXB-BOM d+7 adults=1 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=1&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 508 | `offers` | offers DXB-BOM d+7 adults=1 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=1&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 509 | `offers` | offers DXB-BOM d+7 adults=2 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=2` | 200 | 200 | `application/json` | identical |  |
| 510 | `offers` | offers DXB-BOM d+7 adults=2 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=2&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 511 | `offers` | offers DXB-BOM d+7 adults=2 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=2&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 512 | `offers` | offers DXB-BOM d+7 adults=2 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=2&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 513 | `offers` | offers DXB-BOM d+7 adults=2 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 514 | `offers` | offers DXB-BOM d+7 adults=5 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=5` | 200 | 200 | `application/json` | identical |  |
| 515 | `offers` | offers DXB-BOM d+7 adults=5 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=5&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 516 | `offers` | offers DXB-BOM d+7 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 517 | `offers` | offers DXB-BOM d+7 adults=5 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 518 | `offers` | offers DXB-BOM d+7 adults=5 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=5&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 519 | `offers` | offers DXB-BOM d+7 adults=9 cabin=any | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=9` | 200 | 200 | `application/json` | identical |  |
| 520 | `offers` | offers DXB-BOM d+7 adults=9 cabin=ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=9&cabin=ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 521 | `offers` | offers DXB-BOM d+7 adults=9 cabin=PREMIUM_ECONOMY | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=9&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 522 | `offers` | offers DXB-BOM d+7 adults=9 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=9&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 523 | `offers` | offers DXB-BOM d+7 adults=9 cabin=FIRST | `offers?origin=DXB&destination=BOM&date=2026-11-12&adults=9&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 524 | `offers` | offers DXB-BOM d+15 defaults | `offers?origin=DXB&destination=BOM&date=2026-11-20` | 200 | 200 | `application/json` | identical |  |
| 525 | `offers` | offers DXB-BOM d+29 defaults | `offers?origin=DXB&destination=BOM&date=2026-12-04` | 200 | 200 | `application/json` | identical |  |
| 526 | `offers` | offers BOM-DXB d+0 defaults | `offers?origin=BOM&destination=DXB&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 527 | `offers` | offers NCE-LHR d+0 adults=0 out of range | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=0` | 400 | 400 | `application/problem+json` | identical |  |
| 528 | `offers` | offers NCE-LHR d+0 adults=10 out of range | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=10` | 400 | 400 | `application/problem+json` | identical |  |
| 529 | `offers` | offers NCE-LHR d+0 adults=-1 out of range | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=-1` | 400 | 400 | `application/problem+json` | identical |  |
| 530 | `offers` | offers CDG-JFK d+0 adults=0 out of range | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=0` | 400 | 400 | `application/problem+json` | identical |  |
| 531 | `offers` | offers CDG-JFK d+0 adults=10 out of range | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=10` | 400 | 400 | `application/problem+json` | identical |  |
| 532 | `offers` | offers CDG-JFK d+0 adults=-1 out of range | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=-1` | 400 | 400 | `application/problem+json` | identical |  |
| 533 | `offers` | offers sold out NCE-LHR d+0 adults=5 cabin=PREMIUM_ECONOMY | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=5&cabin=PREMIUM_ECONOMY` | 200 | 200 | `application/json` | identical |  |
| 534 | `offers` | offers sold out NCE-CDG d+0 adults=4 cabin=BUSINESS | `offers?origin=NCE&destination=CDG&date=2026-11-05&adults=4&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 535 | `offers` | offers sold out CDG-JFK d+0 adults=3 cabin=FIRST | `offers?origin=CDG&destination=JFK&date=2026-11-05&adults=3&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 536 | `offers` | offers sold out LHR-SIN d+0 adults=2 cabin=FIRST | `offers?origin=LHR&destination=SIN&date=2026-11-05&adults=2&cabin=FIRST` | 200 | 200 | `application/json` | identical |  |
| 537 | `offers` | offers sold out MAD-FRA d+0 adults=4 cabin=BUSINESS | `offers?origin=MAD&destination=FRA&date=2026-11-05&adults=4&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 538 | `offers` | offers sold out AMS-LIS d+0 adults=5 cabin=BUSINESS | `offers?origin=AMS&destination=LIS&date=2026-11-05&adults=5&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 539 | `offers` | offers sold out FCO-MUC d+0 adults=4 cabin=BUSINESS | `offers?origin=FCO&destination=MUC&date=2026-11-05&adults=4&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 540 | `offers` | offers sold out BCN-DXB d+0 adults=6 cabin=BUSINESS | `offers?origin=BCN&destination=DXB&date=2026-11-05&adults=6&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 541 | `offers` | offers sold out DXB-BOM d+0 adults=8 cabin=BUSINESS | `offers?origin=DXB&destination=BOM&date=2026-11-05&adults=8&cabin=BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 542 | `offers` | offers NCE-LHR d+30 outside window | `offers?origin=NCE&destination=LHR&date=2026-12-05` | 200 | 200 | `application/json` | identical |  |
| 543 | `offers` | offers NCE-LHR d-1 before window | `offers?origin=NCE&destination=LHR&date=2026-11-04` | 200 | 200 | `application/json` | identical |  |
| 544 | `offers` | offers lowercase iata | `offers?origin=nce&destination=lhr&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 545 | `offers` | offers padded adults and cabin | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=%202&cabin=%20BUSINESS` | 200 | 200 | `application/json` | identical |  |
| 546 | `offers` | offers empty adults and cabin | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=&cabin=` | 200 | 200 | `application/json` | identical |  |
| 547 | `offers` | offers adults non numeric | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=x` | 400 | 400 | `application/problem+json` | identical |  |
| 548 | `offers` | offers adults decimal | `offers?origin=NCE&destination=LHR&date=2026-11-05&adults=1.5` | 400 | 400 | `application/problem+json` | identical |  |
| 549 | `offers` | offers cabin unknown | `offers?origin=NCE&destination=LHR&date=2026-11-05&cabin=COACH` | 400 | 400 | `application/problem+json` | identical |  |
| 550 | `offers` | offers cabin lowercase | `offers?origin=NCE&destination=LHR&date=2026-11-05&cabin=economy` | 400 | 400 | `application/problem+json` | identical |  |
| 551 | `offers` | offers unserved route NCE-JFK | `offers?origin=NCE&destination=JFK&date=2026-11-05` | 200 | 200 | `application/json` | identical |  |
| 552 | `offers` | offers unknown origin | `offers?origin=QQQ&destination=LHR&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 553 | `offers` | offers unknown destination | `offers?origin=NCE&destination=QQQ&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 554 | `offers` | offers same origin and destination | `offers?origin=NCE&destination=NCE&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 555 | `offers` | offers missing origin | `offers?destination=LHR&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 556 | `offers` | offers missing destination | `offers?origin=NCE&date=2026-11-05` | 400 | 400 | `application/problem+json` | identical |  |
| 557 | `offers` | offers missing date | `offers?origin=NCE&destination=LHR` | 400 | 400 | `application/problem+json` | identical |  |
| 558 | `offers` | offers no params | `offers` | 400 | 400 | `application/problem+json` | identical |  |
| 559 | `offers` | offers malformed date word | `offers?origin=NCE&destination=LHR&date=bad` | 400 | 400 | `application/problem+json` | identical |  |
| 560 | `offers` | offers malformed date feb 30 | `offers?origin=NCE&destination=LHR&date=2026-02-30` | 400 | 400 | `application/problem+json` | identical |  |
| 561 | `travelers/{id}` | traveler first | `travelers/1` | 200 | 200 | `application/json` | identical |  |
| 562 | `travelers/{id}` | traveler second | `travelers/2` | 200 | 200 | `application/json` | identical |  |
| 563 | `travelers/{id}` | traveler middle | `travelers/25` | 200 | 200 | `application/json` | identical |  |
| 564 | `travelers/{id}` | traveler last | `travelers/50` | 200 | 200 | `application/json` | identical |  |
| 565 | `travelers/{id}` | traveler missing after last | `travelers/51` | 404 | 404 | `application/problem+json` | identical |  |
| 566 | `travelers/{id}` | traveler zero | `travelers/0` | 404 | 404 | `application/problem+json` | identical |  |
| 567 | `travelers/{id}` | traveler non numeric | `travelers/abc` | 400 | 400 | `application/problem+json` | identical |  |
| 568 | `travelers` | travelers lastName A | `travelers?lastName=A` | 200 | 200 | `application/json` | identical |  |
| 569 | `travelers` | travelers lastName B | `travelers?lastName=B` | 200 | 200 | `application/json` | identical |  |
| 570 | `travelers` | travelers lastName C | `travelers?lastName=C` | 200 | 200 | `application/json` | identical |  |
| 571 | `travelers` | travelers lastName D | `travelers?lastName=D` | 200 | 200 | `application/json` | identical |  |
| 572 | `travelers` | travelers lastName E | `travelers?lastName=E` | 200 | 200 | `application/json` | identical |  |
| 573 | `travelers` | travelers lastName F | `travelers?lastName=F` | 200 | 200 | `application/json` | identical |  |
| 574 | `travelers` | travelers lastName G | `travelers?lastName=G` | 200 | 200 | `application/json` | identical |  |
| 575 | `travelers` | travelers lastName H | `travelers?lastName=H` | 200 | 200 | `application/json` | identical |  |
| 576 | `travelers` | travelers lastName I | `travelers?lastName=I` | 200 | 200 | `application/json` | identical |  |
| 577 | `travelers` | travelers lastName J | `travelers?lastName=J` | 200 | 200 | `application/json` | identical |  |
| 578 | `travelers` | travelers lastName K | `travelers?lastName=K` | 200 | 200 | `application/json` | identical |  |
| 579 | `travelers` | travelers lastName L | `travelers?lastName=L` | 200 | 200 | `application/json` | identical |  |
| 580 | `travelers` | travelers lastName M | `travelers?lastName=M` | 200 | 200 | `application/json` | identical |  |
| 581 | `travelers` | travelers lastName N | `travelers?lastName=N` | 200 | 200 | `application/json` | identical |  |
| 582 | `travelers` | travelers lastName O | `travelers?lastName=O` | 200 | 200 | `application/json` | identical |  |
| 583 | `travelers` | travelers lastName P | `travelers?lastName=P` | 200 | 200 | `application/json` | identical |  |
| 584 | `travelers` | travelers lastName Q | `travelers?lastName=Q` | 200 | 200 | `application/json` | identical |  |
| 585 | `travelers` | travelers lastName R | `travelers?lastName=R` | 200 | 200 | `application/json` | identical |  |
| 586 | `travelers` | travelers lastName S | `travelers?lastName=S` | 200 | 200 | `application/json` | identical |  |
| 587 | `travelers` | travelers lastName T | `travelers?lastName=T` | 200 | 200 | `application/json` | identical |  |
| 588 | `travelers` | travelers lastName U | `travelers?lastName=U` | 200 | 200 | `application/json` | identical |  |
| 589 | `travelers` | travelers lastName V | `travelers?lastName=V` | 200 | 200 | `application/json` | identical |  |
| 590 | `travelers` | travelers lastName W | `travelers?lastName=W` | 200 | 200 | `application/json` | identical |  |
| 591 | `travelers` | travelers lastName X | `travelers?lastName=X` | 200 | 200 | `application/json` | identical |  |
| 592 | `travelers` | travelers lastName Y | `travelers?lastName=Y` | 200 | 200 | `application/json` | identical |  |
| 593 | `travelers` | travelers lastName Z | `travelers?lastName=Z` | 200 | 200 | `application/json` | identical |  |
| 594 | `travelers` | travelers lastName Mar | `travelers?lastName=Mar` | 200 | 200 | `application/json` | identical |  |
| 595 | `travelers` | travelers lastName mar lowercase | `travelers?lastName=mar` | 200 | 200 | `application/json` | identical |  |
| 596 | `travelers` | travelers lastName padded Mar | `travelers?lastName=%20Mar%20` | 200 | 200 | `application/json` | identical |  |
| 597 | `travelers` | travelers lastName unknown Zzz | `travelers?lastName=Zzz` | 200 | 200 | `application/json` | identical |  |
| 598 | `travelers` | travelers lastName percent | `travelers?lastName=%25` | 200 | 200 | `application/json` | identical |  |
| 599 | `travelers` | travelers lastName underscore | `travelers?lastName=_` | 200 | 200 | `application/json` | identical |  |
| 600 | `travelers` | travelers lastName empty | `travelers?lastName=` | 400 | 400 | `application/problem+json` | identical |  |
| 601 | `travelers` | travelers lastName blank | `travelers?lastName=%20` | 400 | 400 | `application/problem+json` | identical |  |
| 602 | `travelers` | travelers missing lastName | `travelers` | 400 | 400 | `application/problem+json` | identical |  |
| 603 | `bookings/{recordLocator}` | booking QX7K2M | `bookings/QX7K2M` | 200 | 200 | `application/json` | identical |  |
| 604 | `bookings/{recordLocator}` | booking lowercase qx7k2m | `bookings/qx7k2m` | 200 | 200 | `application/json` | identical |  |
| 605 | `bookings/{recordLocator}` | booking padded qx7k2m | `bookings/%20qx7k2m` | 200 | 200 | `application/json` | identical |  |
| 606 | `bookings/{recordLocator}` | booking missing ZZZZZZ | `bookings/ZZZZZZ` | 404 | 404 | `application/problem+json` | identical |  |
| 607 | `bookings/{recordLocator}` | booking too short QX7 | `bookings/QX7` | 404 | 404 | `application/problem+json` | identical |  |
| 608 | `bookings/{recordLocator}` | booking 25KPRU | `bookings/25KPRU` | 200 | 200 | `application/json` | identical |  |
| 609 | `bookings/{recordLocator}` | booking 29YZEU | `bookings/29YZEU` | 200 | 200 | `application/json` | identical |  |
| 610 | `bookings/{recordLocator}` | booking 29ZXDH | `bookings/29ZXDH` | 200 | 200 | `application/json` | identical |  |
| 611 | `bookings/{recordLocator}` | booking 2PQ2R7 | `bookings/2PQ2R7` | 200 | 200 | `application/json` | identical |  |
| 612 | `bookings/{recordLocator}` | booking 2PWX57 | `bookings/2PWX57` | 200 | 200 | `application/json` | identical |  |
| 613 | `bookings/{recordLocator}` | booking 33SUH9 | `bookings/33SUH9` | 200 | 200 | `application/json` | identical |  |
| 614 | `bookings/{recordLocator}` | booking 3BMY3Y | `bookings/3BMY3Y` | 200 | 200 | `application/json` | identical |  |
| 615 | `bookings/{recordLocator}` | booking 48VP9H | `bookings/48VP9H` | 200 | 200 | `application/json` | identical |  |
| 616 | `bookings/{recordLocator}` | booking 4AY7KB | `bookings/4AY7KB` | 200 | 200 | `application/json` | identical |  |
| 617 | `bookings/{recordLocator}` | booking 4CQ6NQ | `bookings/4CQ6NQ` | 200 | 200 | `application/json` | identical |  |
| 618 | `bookings/{recordLocator}` | booking 4F9QC9 | `bookings/4F9QC9` | 200 | 200 | `application/json` | identical |  |
| 619 | `bookings/{recordLocator}` | booking 4LJ8WY | `bookings/4LJ8WY` | 200 | 200 | `application/json` | identical |  |
| 620 | `bookings/{recordLocator}` | booking 4MNZD3 | `bookings/4MNZD3` | 200 | 200 | `application/json` | identical |  |
| 621 | `bookings/{recordLocator}` | booking 4TC6UJ | `bookings/4TC6UJ` | 200 | 200 | `application/json` | identical |  |
| 622 | `bookings/{recordLocator}` | booking 4VCFBR | `bookings/4VCFBR` | 200 | 200 | `application/json` | identical |  |
| 623 | `bookings/{recordLocator}` | booking 59E5XL | `bookings/59E5XL` | 200 | 200 | `application/json` | identical |  |
| 624 | `bookings/{recordLocator}` | booking 5DX4YN | `bookings/5DX4YN` | 200 | 200 | `application/json` | identical |  |
| 625 | `bookings/{recordLocator}` | booking 5EXMLY | `bookings/5EXMLY` | 200 | 200 | `application/json` | identical |  |
| 626 | `bookings/{recordLocator}` | booking 5HD2CH | `bookings/5HD2CH` | 200 | 200 | `application/json` | identical |  |
| 627 | `bookings/{recordLocator}` | booking 5HY2WG | `bookings/5HY2WG` | 200 | 200 | `application/json` | identical |  |
| 628 | `bookings/{recordLocator}` | booking 5J3LCB | `bookings/5J3LCB` | 200 | 200 | `application/json` | identical |  |
| 629 | `bookings/{recordLocator}` | booking 5M3NCT | `bookings/5M3NCT` | 200 | 200 | `application/json` | identical |  |
| 630 | `bookings/{recordLocator}` | booking 5S3SGV | `bookings/5S3SGV` | 200 | 200 | `application/json` | identical |  |
| 631 | `bookings/{recordLocator}` | booking 5TER9A | `bookings/5TER9A` | 200 | 200 | `application/json` | identical |  |
| 632 | `bookings/{recordLocator}` | booking 5Y99GP | `bookings/5Y99GP` | 200 | 200 | `application/json` | identical |  |
| 633 | `bookings/{recordLocator}` | booking 5YWCJY | `bookings/5YWCJY` | 200 | 200 | `application/json` | identical |  |
| 634 | `bookings/{recordLocator}` | booking 625VXQ | `bookings/625VXQ` | 200 | 200 | `application/json` | identical |  |
| 635 | `bookings/{recordLocator}` | booking 63YHBW | `bookings/63YHBW` | 200 | 200 | `application/json` | identical |  |
| 636 | `bookings/{recordLocator}` | booking 64297L | `bookings/64297L` | 200 | 200 | `application/json` | identical |  |
| 637 | `bookings/{recordLocator}` | booking 6KPKCG | `bookings/6KPKCG` | 200 | 200 | `application/json` | identical |  |
| 638 | `bookings/{recordLocator}` | booking 6LDJLA | `bookings/6LDJLA` | 200 | 200 | `application/json` | identical |  |
| 639 | `bookings/{recordLocator}` | booking 6UPXR7 | `bookings/6UPXR7` | 200 | 200 | `application/json` | identical |  |
| 640 | `bookings/{recordLocator}` | booking 6ZJQPK | `bookings/6ZJQPK` | 200 | 200 | `application/json` | identical |  |
| 641 | `bookings/{recordLocator}` | booking 75AHJA | `bookings/75AHJA` | 200 | 200 | `application/json` | identical |  |
| 642 | `bookings/{recordLocator}` | booking 76MTWE | `bookings/76MTWE` | 200 | 200 | `application/json` | identical |  |
| 643 | `bookings/{recordLocator}` | booking 7ASXAF | `bookings/7ASXAF` | 200 | 200 | `application/json` | identical |  |
| 644 | `bookings/{recordLocator}` | booking 7FZMQG | `bookings/7FZMQG` | 200 | 200 | `application/json` | identical |  |
| 645 | `bookings/{recordLocator}` | booking 7QGBEA | `bookings/7QGBEA` | 200 | 200 | `application/json` | identical |  |
| 646 | `bookings/{recordLocator}` | booking 7SAM7T | `bookings/7SAM7T` | 200 | 200 | `application/json` | identical |  |
| 647 | `bookings/{recordLocator}` | booking 7SQF99 | `bookings/7SQF99` | 200 | 200 | `application/json` | identical |  |
| 648 | `bookings/{recordLocator}` | booking 7TE7YQ | `bookings/7TE7YQ` | 200 | 200 | `application/json` | identical |  |
| 649 | `bookings/{recordLocator}` | booking 7XW4BT | `bookings/7XW4BT` | 200 | 200 | `application/json` | identical |  |
| 650 | `bookings/{recordLocator}` | booking 825LH3 | `bookings/825LH3` | 200 | 200 | `application/json` | identical |  |
| 651 | `bookings/{recordLocator}` | booking 8GCBQZ | `bookings/8GCBQZ` | 200 | 200 | `application/json` | identical |  |
| 652 | `bookings/{recordLocator}` | booking 8LXF9G | `bookings/8LXF9G` | 200 | 200 | `application/json` | identical |  |
| 653 | `bookings/{recordLocator}` | booking 8NPWCX | `bookings/8NPWCX` | 200 | 200 | `application/json` | identical |  |
| 654 | `bookings/{recordLocator}` | booking 8NRZJJ | `bookings/8NRZJJ` | 200 | 200 | `application/json` | identical |  |
| 655 | `bookings/{recordLocator}` | booking 8UC358 | `bookings/8UC358` | 200 | 200 | `application/json` | identical |  |
| 656 | `bookings/{recordLocator}` | booking 955D34 | `bookings/955D34` | 200 | 200 | `application/json` | identical |  |
| 657 | `bookings/{recordLocator}` | booking 95Q858 | `bookings/95Q858` | 200 | 200 | `application/json` | identical |  |
| 658 | `bookings/{recordLocator}` | booking 9C77DS | `bookings/9C77DS` | 200 | 200 | `application/json` | identical |  |
| 659 | `bookings/{recordLocator}` | booking 9EPCJY | `bookings/9EPCJY` | 200 | 200 | `application/json` | identical |  |
| 660 | `bookings/{recordLocator}` | booking 9R7AAK | `bookings/9R7AAK` | 200 | 200 | `application/json` | identical |  |
| 661 | `bookings/{recordLocator}` | booking 9UFCAM | `bookings/9UFCAM` | 200 | 200 | `application/json` | identical |  |
| 662 | `bookings/{recordLocator}` | booking A3A29L | `bookings/A3A29L` | 200 | 200 | `application/json` | identical |  |
| 663 | `bookings/{recordLocator}` | booking A8S292 | `bookings/A8S292` | 200 | 200 | `application/json` | identical |  |
| 664 | `bookings/{recordLocator}` | booking B49KB8 | `bookings/B49KB8` | 200 | 200 | `application/json` | identical |  |
| 665 | `bookings/{recordLocator}` | booking B6X8S8 | `bookings/B6X8S8` | 200 | 200 | `application/json` | identical |  |
| 666 | `bookings/{recordLocator}` | booking B8FKEC | `bookings/B8FKEC` | 200 | 200 | `application/json` | identical |  |
| 667 | `bookings/{recordLocator}` | booking BAMEMQ | `bookings/BAMEMQ` | 200 | 200 | `application/json` | identical |  |
| 668 | `bookings/{recordLocator}` | booking BBKXDT | `bookings/BBKXDT` | 200 | 200 | `application/json` | identical |  |
| 669 | `bookings/{recordLocator}` | booking BM2XPW | `bookings/BM2XPW` | 200 | 200 | `application/json` | identical |  |
| 670 | `bookings/{recordLocator}` | booking BN3QD8 | `bookings/BN3QD8` | 200 | 200 | `application/json` | identical |  |
| 671 | `bookings/{recordLocator}` | booking BNZ384 | `bookings/BNZ384` | 200 | 200 | `application/json` | identical |  |
| 672 | `bookings/{recordLocator}` | booking BX9E3B | `bookings/BX9E3B` | 200 | 200 | `application/json` | identical |  |
| 673 | `bookings/{recordLocator}` | booking BXN4PD | `bookings/BXN4PD` | 200 | 200 | `application/json` | identical |  |
| 674 | `bookings/{recordLocator}` | booking BXVRG7 | `bookings/BXVRG7` | 200 | 200 | `application/json` | identical |  |
| 675 | `bookings/{recordLocator}` | booking CB2475 | `bookings/CB2475` | 200 | 200 | `application/json` | identical |  |
| 676 | `bookings/{recordLocator}` | booking CEYCKT | `bookings/CEYCKT` | 200 | 200 | `application/json` | identical |  |
| 677 | `bookings/{recordLocator}` | booking CKV8MK | `bookings/CKV8MK` | 200 | 200 | `application/json` | identical |  |
| 678 | `bookings/{recordLocator}` | booking CQFUZZ | `bookings/CQFUZZ` | 200 | 200 | `application/json` | identical |  |
| 679 | `bookings/{recordLocator}` | booking CRZ2RC | `bookings/CRZ2RC` | 200 | 200 | `application/json` | identical |  |
| 680 | `bookings/{recordLocator}` | booking CSLSTN | `bookings/CSLSTN` | 200 | 200 | `application/json` | identical |  |
| 681 | `bookings/{recordLocator}` | booking CTEGPF | `bookings/CTEGPF` | 200 | 200 | `application/json` | identical |  |
| 682 | `bookings/{recordLocator}` | booking CUPSU5 | `bookings/CUPSU5` | 200 | 200 | `application/json` | identical |  |
| 683 | `bookings/{recordLocator}` | booking CUTBW3 | `bookings/CUTBW3` | 200 | 200 | `application/json` | identical |  |
| 684 | `bookings/{recordLocator}` | booking DSLE9U | `bookings/DSLE9U` | 200 | 200 | `application/json` | identical |  |
| 685 | `bookings/{recordLocator}` | booking DUWAFA | `bookings/DUWAFA` | 200 | 200 | `application/json` | identical |  |
| 686 | `bookings/{recordLocator}` | booking DVF855 | `bookings/DVF855` | 200 | 200 | `application/json` | identical |  |
| 687 | `bookings/{recordLocator}` | booking E2P6ST | `bookings/E2P6ST` | 200 | 200 | `application/json` | identical |  |
| 688 | `bookings/{recordLocator}` | booking E3NEMW | `bookings/E3NEMW` | 200 | 200 | `application/json` | identical |  |
| 689 | `bookings/{recordLocator}` | booking E9KAHG | `bookings/E9KAHG` | 200 | 200 | `application/json` | identical |  |
| 690 | `bookings/{recordLocator}` | booking E9QGXD | `bookings/E9QGXD` | 200 | 200 | `application/json` | identical |  |
| 691 | `bookings/{recordLocator}` | booking E9T2CU | `bookings/E9T2CU` | 200 | 200 | `application/json` | identical |  |
| 692 | `bookings/{recordLocator}` | booking EKR4TC | `bookings/EKR4TC` | 200 | 200 | `application/json` | identical |  |
| 693 | `bookings/{recordLocator}` | booking ETVB94 | `bookings/ETVB94` | 200 | 200 | `application/json` | identical |  |
| 694 | `bookings/{recordLocator}` | booking FATNAW | `bookings/FATNAW` | 200 | 200 | `application/json` | identical |  |
| 695 | `bookings/{recordLocator}` | booking FN6R6B | `bookings/FN6R6B` | 200 | 200 | `application/json` | identical |  |
| 696 | `bookings/{recordLocator}` | booking G5XM64 | `bookings/G5XM64` | 200 | 200 | `application/json` | identical |  |
| 697 | `bookings/{recordLocator}` | booking G6J466 | `bookings/G6J466` | 200 | 200 | `application/json` | identical |  |
| 698 | `bookings/{recordLocator}` | booking G9LDCG | `bookings/G9LDCG` | 200 | 200 | `application/json` | identical |  |
| 699 | `bookings/{recordLocator}` | booking GZDQMT | `bookings/GZDQMT` | 200 | 200 | `application/json` | identical |  |
| 700 | `bookings/{recordLocator}` | booking H5D2E4 | `bookings/H5D2E4` | 200 | 200 | `application/json` | identical |  |
| 701 | `bookings/{recordLocator}` | booking H5DE5P | `bookings/H5DE5P` | 200 | 200 | `application/json` | identical |  |
| 702 | `bookings/{recordLocator}` | booking H8MRSA | `bookings/H8MRSA` | 200 | 200 | `application/json` | identical |  |
| 703 | `bookings/{recordLocator}` | booking HB2KZ7 | `bookings/HB2KZ7` | 200 | 200 | `application/json` | identical |  |
| 704 | `bookings/{recordLocator}` | booking HKCQ62 | `bookings/HKCQ62` | 200 | 200 | `application/json` | identical |  |
| 705 | `bookings/{recordLocator}` | booking HPGYB5 | `bookings/HPGYB5` | 200 | 200 | `application/json` | identical |  |
| 706 | `bookings/{recordLocator}` | booking HR8NR4 | `bookings/HR8NR4` | 200 | 200 | `application/json` | identical |  |
| 707 | `bookings/{recordLocator}` | booking JDUQ3U | `bookings/JDUQ3U` | 200 | 200 | `application/json` | identical |  |
| 708 | `bookings/{recordLocator}` | booking JYX5CF | `bookings/JYX5CF` | 200 | 200 | `application/json` | identical |  |
| 709 | `bookings/{recordLocator}` | booking K3ZJ6R | `bookings/K3ZJ6R` | 200 | 200 | `application/json` | identical |  |
| 710 | `bookings/{recordLocator}` | booking KD5XR2 | `bookings/KD5XR2` | 200 | 200 | `application/json` | identical |  |
| 711 | `bookings/{recordLocator}` | booking KL72N6 | `bookings/KL72N6` | 200 | 200 | `application/json` | identical |  |
| 712 | `bookings/{recordLocator}` | booking KNNMR7 | `bookings/KNNMR7` | 200 | 200 | `application/json` | identical |  |
| 713 | `bookings/{recordLocator}` | booking KWLX26 | `bookings/KWLX26` | 200 | 200 | `application/json` | identical |  |
| 714 | `bookings/{recordLocator}` | booking L24QEG | `bookings/L24QEG` | 200 | 200 | `application/json` | identical |  |
| 715 | `bookings/{recordLocator}` | booking L3JJX6 | `bookings/L3JJX6` | 200 | 200 | `application/json` | identical |  |
| 716 | `bookings/{recordLocator}` | booking LEYNR8 | `bookings/LEYNR8` | 200 | 200 | `application/json` | identical |  |
| 717 | `bookings/{recordLocator}` | booking LJZ7V5 | `bookings/LJZ7V5` | 200 | 200 | `application/json` | identical |  |
| 718 | `bookings/{recordLocator}` | booking LL5NAH | `bookings/LL5NAH` | 200 | 200 | `application/json` | identical |  |
| 719 | `bookings/{recordLocator}` | booking LLM3W4 | `bookings/LLM3W4` | 200 | 200 | `application/json` | identical |  |
| 720 | `bookings/{recordLocator}` | booking LY32A8 | `bookings/LY32A8` | 200 | 200 | `application/json` | identical |  |
| 721 | `bookings/{recordLocator}` | booking MKMW6B | `bookings/MKMW6B` | 200 | 200 | `application/json` | identical |  |
| 722 | `bookings/{recordLocator}` | booking MVUV8N | `bookings/MVUV8N` | 200 | 200 | `application/json` | identical |  |
| 723 | `bookings/{recordLocator}` | booking MWZ5WE | `bookings/MWZ5WE` | 200 | 200 | `application/json` | identical |  |
| 724 | `bookings/{recordLocator}` | booking MXZZ9H | `bookings/MXZZ9H` | 200 | 200 | `application/json` | identical |  |
| 725 | `bookings/{recordLocator}` | booking MY884J | `bookings/MY884J` | 200 | 200 | `application/json` | identical |  |
| 726 | `bookings/{recordLocator}` | booking N5ZYJ9 | `bookings/N5ZYJ9` | 200 | 200 | `application/json` | identical |  |
| 727 | `bookings/{recordLocator}` | booking NB6UVP | `bookings/NB6UVP` | 200 | 200 | `application/json` | identical |  |
| 728 | `bookings/{recordLocator}` | booking NG2GSJ | `bookings/NG2GSJ` | 200 | 200 | `application/json` | identical |  |
| 729 | `bookings/{recordLocator}` | booking NKPZ8P | `bookings/NKPZ8P` | 200 | 200 | `application/json` | identical |  |
| 730 | `bookings/{recordLocator}` | booking NNVBRA | `bookings/NNVBRA` | 200 | 200 | `application/json` | identical |  |
| 731 | `bookings/{recordLocator}` | booking NS4MGZ | `bookings/NS4MGZ` | 200 | 200 | `application/json` | identical |  |
| 732 | `bookings/{recordLocator}` | booking NV4U83 | `bookings/NV4U83` | 200 | 200 | `application/json` | identical |  |
| 733 | `bookings/{recordLocator}` | booking P4EU4L | `bookings/P4EU4L` | 200 | 200 | `application/json` | identical |  |
| 734 | `bookings/{recordLocator}` | booking P64PDA | `bookings/P64PDA` | 200 | 200 | `application/json` | identical |  |
| 735 | `bookings/{recordLocator}` | booking PX899K | `bookings/PX899K` | 200 | 200 | `application/json` | identical |  |
| 736 | `bookings/{recordLocator}` | booking PYNJES | `bookings/PYNJES` | 200 | 200 | `application/json` | identical |  |
| 737 | `bookings/{recordLocator}` | booking Q2XB7V | `bookings/Q2XB7V` | 200 | 200 | `application/json` | identical |  |
| 738 | `bookings/{recordLocator}` | booking QEAEWU | `bookings/QEAEWU` | 200 | 200 | `application/json` | identical |  |
| 739 | `bookings/{recordLocator}` | booking QFERDF | `bookings/QFERDF` | 200 | 200 | `application/json` | identical |  |
| 740 | `bookings/{recordLocator}` | booking R724HP | `bookings/R724HP` | 200 | 200 | `application/json` | identical |  |
| 741 | `bookings/{recordLocator}` | booking R88KDF | `bookings/R88KDF` | 200 | 200 | `application/json` | identical |  |
| 742 | `bookings/{recordLocator}` | booking R8E65K | `bookings/R8E65K` | 200 | 200 | `application/json` | identical |  |
| 743 | `bookings/{recordLocator}` | booking RDDPD9 | `bookings/RDDPD9` | 200 | 200 | `application/json` | identical |  |
| 744 | `bookings/{recordLocator}` | booking RGDKHH | `bookings/RGDKHH` | 200 | 200 | `application/json` | identical |  |
| 745 | `bookings/{recordLocator}` | booking RHD99W | `bookings/RHD99W` | 200 | 200 | `application/json` | identical |  |
| 746 | `bookings/{recordLocator}` | booking SKUCNS | `bookings/SKUCNS` | 200 | 200 | `application/json` | identical |  |
| 747 | `bookings/{recordLocator}` | booking SLFZL3 | `bookings/SLFZL3` | 200 | 200 | `application/json` | identical |  |
| 748 | `bookings/{recordLocator}` | booking SLUVAK | `bookings/SLUVAK` | 200 | 200 | `application/json` | identical |  |
| 749 | `bookings/{recordLocator}` | booking SPPJFP | `bookings/SPPJFP` | 200 | 200 | `application/json` | identical |  |
| 750 | `bookings/{recordLocator}` | booking SSA5JV | `bookings/SSA5JV` | 200 | 200 | `application/json` | identical |  |
| 751 | `bookings/{recordLocator}` | booking SSRKT9 | `bookings/SSRKT9` | 200 | 200 | `application/json` | identical |  |
| 752 | `bookings/{recordLocator}` | booking SXT2VS | `bookings/SXT2VS` | 200 | 200 | `application/json` | identical |  |
| 753 | `bookings/{recordLocator}` | booking SXT56S | `bookings/SXT56S` | 200 | 200 | `application/json` | identical |  |
| 754 | `bookings/{recordLocator}` | booking T75TY2 | `bookings/T75TY2` | 200 | 200 | `application/json` | identical |  |
| 755 | `bookings/{recordLocator}` | booking T7CJSP | `bookings/T7CJSP` | 200 | 200 | `application/json` | identical |  |
| 756 | `bookings/{recordLocator}` | booking T7KU6Q | `bookings/T7KU6Q` | 200 | 200 | `application/json` | identical |  |
| 757 | `bookings/{recordLocator}` | booking T9ZTHZ | `bookings/T9ZTHZ` | 200 | 200 | `application/json` | identical |  |
| 758 | `bookings/{recordLocator}` | booking TGYB9U | `bookings/TGYB9U` | 200 | 200 | `application/json` | identical |  |
| 759 | `bookings/{recordLocator}` | booking TLL6FL | `bookings/TLL6FL` | 200 | 200 | `application/json` | identical |  |
| 760 | `bookings/{recordLocator}` | booking TPLVEL | `bookings/TPLVEL` | 200 | 200 | `application/json` | identical |  |
| 761 | `bookings/{recordLocator}` | booking U5HFTR | `bookings/U5HFTR` | 200 | 200 | `application/json` | identical |  |
| 762 | `bookings/{recordLocator}` | booking UAUXBR | `bookings/UAUXBR` | 200 | 200 | `application/json` | identical |  |
| 763 | `bookings/{recordLocator}` | booking UCP8BW | `bookings/UCP8BW` | 200 | 200 | `application/json` | identical |  |
| 764 | `bookings/{recordLocator}` | booking UT4UV6 | `bookings/UT4UV6` | 200 | 200 | `application/json` | identical |  |
| 765 | `bookings/{recordLocator}` | booking V2L39Q | `bookings/V2L39Q` | 200 | 200 | `application/json` | identical |  |
| 766 | `bookings/{recordLocator}` | booking V67XEB | `bookings/V67XEB` | 200 | 200 | `application/json` | identical |  |
| 767 | `bookings/{recordLocator}` | booking VC5DHA | `bookings/VC5DHA` | 200 | 200 | `application/json` | identical |  |
| 768 | `bookings/{recordLocator}` | booking VCB3HZ | `bookings/VCB3HZ` | 200 | 200 | `application/json` | identical |  |
| 769 | `bookings/{recordLocator}` | booking VCRXGC | `bookings/VCRXGC` | 200 | 200 | `application/json` | identical |  |
| 770 | `bookings/{recordLocator}` | booking VRFS7P | `bookings/VRFS7P` | 200 | 200 | `application/json` | identical |  |
| 771 | `bookings/{recordLocator}` | booking VVRY7N | `bookings/VVRY7N` | 200 | 200 | `application/json` | identical |  |
| 772 | `bookings/{recordLocator}` | booking VYKN32 | `bookings/VYKN32` | 200 | 200 | `application/json` | identical |  |
| 773 | `bookings/{recordLocator}` | booking VZMZTF | `bookings/VZMZTF` | 200 | 200 | `application/json` | identical |  |
| 774 | `bookings/{recordLocator}` | booking VZUVD2 | `bookings/VZUVD2` | 200 | 200 | `application/json` | identical |  |
| 775 | `bookings/{recordLocator}` | booking W2JW2D | `bookings/W2JW2D` | 200 | 200 | `application/json` | identical |  |
| 776 | `bookings/{recordLocator}` | booking W62ZLT | `bookings/W62ZLT` | 200 | 200 | `application/json` | identical |  |
| 777 | `bookings/{recordLocator}` | booking W6MH7N | `bookings/W6MH7N` | 200 | 200 | `application/json` | identical |  |
| 778 | `bookings/{recordLocator}` | booking WDEDWA | `bookings/WDEDWA` | 200 | 200 | `application/json` | identical |  |
| 779 | `bookings/{recordLocator}` | booking WEJU5Y | `bookings/WEJU5Y` | 200 | 200 | `application/json` | identical |  |
| 780 | `bookings/{recordLocator}` | booking WN8RUA | `bookings/WN8RUA` | 200 | 200 | `application/json` | identical |  |
| 781 | `bookings/{recordLocator}` | booking WQVR5D | `bookings/WQVR5D` | 200 | 200 | `application/json` | identical |  |
| 782 | `bookings/{recordLocator}` | booking WRVEF9 | `bookings/WRVEF9` | 200 | 200 | `application/json` | identical |  |
| 783 | `bookings/{recordLocator}` | booking X49FPZ | `bookings/X49FPZ` | 200 | 200 | `application/json` | identical |  |
| 784 | `bookings/{recordLocator}` | booking X9G7PB | `bookings/X9G7PB` | 200 | 200 | `application/json` | identical |  |
| 785 | `bookings/{recordLocator}` | booking XAVVH2 | `bookings/XAVVH2` | 200 | 200 | `application/json` | identical |  |
| 786 | `bookings/{recordLocator}` | booking XB4L7H | `bookings/XB4L7H` | 200 | 200 | `application/json` | identical |  |
| 787 | `bookings/{recordLocator}` | booking XDXBCK | `bookings/XDXBCK` | 200 | 200 | `application/json` | identical |  |
| 788 | `bookings/{recordLocator}` | booking XHLV6Y | `bookings/XHLV6Y` | 200 | 200 | `application/json` | identical |  |
| 789 | `bookings/{recordLocator}` | booking XU2ZRN | `bookings/XU2ZRN` | 200 | 200 | `application/json` | identical |  |
| 790 | `bookings/{recordLocator}` | booking XVLWN7 | `bookings/XVLWN7` | 200 | 200 | `application/json` | identical |  |
| 791 | `bookings/{recordLocator}` | booking XXRHDQ | `bookings/XXRHDQ` | 200 | 200 | `application/json` | identical |  |
| 792 | `bookings/{recordLocator}` | booking XYGPWV | `bookings/XYGPWV` | 200 | 200 | `application/json` | identical |  |
| 793 | `bookings/{recordLocator}` | booking YH7XMF | `bookings/YH7XMF` | 200 | 200 | `application/json` | identical |  |
| 794 | `bookings/{recordLocator}` | booking YHPEQP | `bookings/YHPEQP` | 200 | 200 | `application/json` | identical |  |
| 795 | `bookings/{recordLocator}` | booking YP8V6T | `bookings/YP8V6T` | 200 | 200 | `application/json` | identical |  |
| 796 | `bookings/{recordLocator}` | booking YPQVUH | `bookings/YPQVUH` | 200 | 200 | `application/json` | identical |  |
| 797 | `bookings/{recordLocator}` | booking YVUWHG | `bookings/YVUWHG` | 200 | 200 | `application/json` | identical |  |
| 798 | `bookings/{recordLocator}` | booking Z63EMR | `bookings/Z63EMR` | 200 | 200 | `application/json` | identical |  |
| 799 | `bookings/{recordLocator}` | booking Z723XX | `bookings/Z723XX` | 200 | 200 | `application/json` | identical |  |
| 800 | `bookings/{recordLocator}` | booking ZDDKE2 | `bookings/ZDDKE2` | 200 | 200 | `application/json` | identical |  |
| 801 | `bookings/{recordLocator}` | booking ZGJWG8 | `bookings/ZGJWG8` | 200 | 200 | `application/json` | identical |  |
| 802 | `bookings/{recordLocator}` | booking ZGXDJQ | `bookings/ZGXDJQ` | 200 | 200 | `application/json` | identical |  |
| 803 | `bookings/{recordLocator}` | booking ZPG3UA | `bookings/ZPG3UA` | 200 | 200 | `application/json` | identical |  |
| 804 | `bookings/{recordLocator}` | booking ZQVYME | `bookings/ZQVYME` | 200 | 200 | `application/json` | identical |  |
| 805 | `bookings/{recordLocator}` | booking ZTNCU2 | `bookings/ZTNCU2` | 200 | 200 | `application/json` | identical |  |
| 806 | `bookings/{recordLocator}` | booking ZWAPQV | `bookings/ZWAPQV` | 200 | 200 | `application/json` | identical |  |
| 807 | `bookings` | bookings lastName A | `bookings?lastName=A` | 200 | 200 | `application/json` | identical |  |
| 808 | `bookings` | bookings lastName B | `bookings?lastName=B` | 200 | 200 | `application/json` | identical |  |
| 809 | `bookings` | bookings lastName C | `bookings?lastName=C` | 200 | 200 | `application/json` | identical |  |
| 810 | `bookings` | bookings lastName D | `bookings?lastName=D` | 200 | 200 | `application/json` | identical |  |
| 811 | `bookings` | bookings lastName E | `bookings?lastName=E` | 200 | 200 | `application/json` | identical |  |
| 812 | `bookings` | bookings lastName F | `bookings?lastName=F` | 200 | 200 | `application/json` | identical |  |
| 813 | `bookings` | bookings lastName G | `bookings?lastName=G` | 200 | 200 | `application/json` | identical |  |
| 814 | `bookings` | bookings lastName H | `bookings?lastName=H` | 200 | 200 | `application/json` | identical |  |
| 815 | `bookings` | bookings lastName I | `bookings?lastName=I` | 200 | 200 | `application/json` | identical |  |
| 816 | `bookings` | bookings lastName J | `bookings?lastName=J` | 200 | 200 | `application/json` | identical |  |
| 817 | `bookings` | bookings lastName K | `bookings?lastName=K` | 200 | 200 | `application/json` | identical |  |
| 818 | `bookings` | bookings lastName L | `bookings?lastName=L` | 200 | 200 | `application/json` | identical |  |
| 819 | `bookings` | bookings lastName M | `bookings?lastName=M` | 200 | 200 | `application/json` | identical |  |
| 820 | `bookings` | bookings lastName N | `bookings?lastName=N` | 200 | 200 | `application/json` | identical |  |
| 821 | `bookings` | bookings lastName O | `bookings?lastName=O` | 200 | 200 | `application/json` | identical |  |
| 822 | `bookings` | bookings lastName P | `bookings?lastName=P` | 200 | 200 | `application/json` | identical |  |
| 823 | `bookings` | bookings lastName Q | `bookings?lastName=Q` | 200 | 200 | `application/json` | identical |  |
| 824 | `bookings` | bookings lastName R | `bookings?lastName=R` | 200 | 200 | `application/json` | identical |  |
| 825 | `bookings` | bookings lastName S | `bookings?lastName=S` | 200 | 200 | `application/json` | identical |  |
| 826 | `bookings` | bookings lastName T | `bookings?lastName=T` | 200 | 200 | `application/json` | identical |  |
| 827 | `bookings` | bookings lastName U | `bookings?lastName=U` | 200 | 200 | `application/json` | identical |  |
| 828 | `bookings` | bookings lastName V | `bookings?lastName=V` | 200 | 200 | `application/json` | identical |  |
| 829 | `bookings` | bookings lastName W | `bookings?lastName=W` | 200 | 200 | `application/json` | identical |  |
| 830 | `bookings` | bookings lastName X | `bookings?lastName=X` | 200 | 200 | `application/json` | identical |  |
| 831 | `bookings` | bookings lastName Y | `bookings?lastName=Y` | 200 | 200 | `application/json` | identical |  |
| 832 | `bookings` | bookings lastName Z | `bookings?lastName=Z` | 200 | 200 | `application/json` | identical |  |
| 833 | `bookings` | bookings lastName Mar | `bookings?lastName=Mar` | 200 | 200 | `application/json` | identical |  |
| 834 | `bookings` | bookings lastName mar lowercase | `bookings?lastName=mar` | 200 | 200 | `application/json` | identical |  |
| 835 | `bookings` | bookings lastName padded Mar | `bookings?lastName=%20Mar` | 200 | 200 | `application/json` | identical |  |
| 836 | `bookings` | bookings lastName unknown Zzz | `bookings?lastName=Zzz` | 200 | 200 | `application/json` | identical |  |
| 837 | `bookings` | bookings lastName percent | `bookings?lastName=%25` | 200 | 200 | `application/json` | identical |  |
| 838 | `bookings` | bookings lastName underscore | `bookings?lastName=_` | 200 | 200 | `application/json` | identical |  |
| 839 | `bookings` | bookings lastName empty | `bookings?lastName=` | 400 | 400 | `application/problem+json` | identical |  |
| 840 | `bookings` | bookings lastName blank | `bookings?lastName=%20` | 400 | 400 | `application/problem+json` | identical |  |
| 841 | `bookings` | bookings missing lastName | `bookings` | 400 | 400 | `application/problem+json` | identical |  |
| 842 | `unknown route` | unknown path | `nope` | 404 | 404 | `application/problem+json` | identical |  |
| 843 | `unknown route` | unknown sub resource | `flights/1/fares` | 404 | 404 | `application/problem+json` | identical |  |
