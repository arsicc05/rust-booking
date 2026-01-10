📅 Mikroservisna aplikacija za zakazivanje termina
1. Uvod

Ovaj projekat predstavlja implementaciju mikroservisne aplikacije za zakazivanje termina (npr. kod lekara, frizera ili konsultanata), realizovane u skladu sa principima servisno orijentisane arhitekture. Sistem omogućava korisnicima registraciju, autentifikaciju, pregled dostupnih termina, zakazivanje i otkazivanje termina, kao i dobijanje potvrde o zakazanom terminu.

Backend sistema je implementiran korišćenjem Rust programskog jezika kroz više nezavisnih mikroservisa, dok je frontend realizovan kao web aplikacija u Next.js-u. Izgled aplikacije nije u fokusu, već funkcionalnost i pravilna primena mikroservisne arhitekture.

2. Arhitektura sistema

Aplikacija je realizovana kao skup nezavisnih mikroservisa koji međusobno komuniciraju putem REST API-ja i asinhronih poruka. Svaki servis poseduje sopstvenu bazu podataka i jasno definisanu odgovornost.

Pregled servisa:

Auth Service

User/Profile Service

Appointment Service

Notification Service

API Gateway (opciono)

3. Opis mikroservisa
3.1 Auth Service

Namena:
Zadužen za autentifikaciju i autorizaciju korisnika.

Funkcionalnosti:

Registracija korisnika

Prijava korisnika

Izdavanje i validacija JWT tokena

Upravljanje ulogama (korisnik, pružalac usluge, administrator)

Tehnologije:

Rust (Axum / Actix)

PostgreSQL

JWT

3.2 User / Profile Service

Namena:
Upravljanje podacima o korisnicima i pružaocima usluga.

Funkcionalnosti:

CRUD operacije nad korisničkim profilima

Čuvanje podataka o lokaciji pružaoca usluge

Upload i prikaz profilnih slika

Tehnologije:

Rust

PostgreSQL

Rad sa slikama (upload i čuvanje metapodataka)

3.3 Appointment Service

Namena:
Upravljanje terminima i njihovim statusima.

Funkcionalnosti:

Kreiranje dostupnih termina

Zakazivanje i otkazivanje termina

Provera dostupnosti termina

Upravljanje statusima (slobodan, zakazan, otkazan)

Napredni koncepti:

Saga pattern za proces zakazivanja termina

Validacija dostupnosti termina u realnom vremenu

Tehnologije:

Rust

PostgreSQL

3.4 Notification Service

Namena:
Slanje notifikacija korisnicima nakon uspešnih ili neuspešnih operacija.

Funkcionalnosti:

Slanje notifikacija o zakazanom ili otkazanom terminu

Generisanje QR koda koji predstavlja potvrdu termina

Napredni koncepti:

Asinhrona komunikacija (message broker)

Obrada događaja (AppointmentCreated, AppointmentCanceled)

Tehnologije:

Rust

Redis / MongoDB

Message broker (NATS ili RabbitMQ)

4. API Gateway

API Gateway predstavlja jedinstvenu ulaznu tačku ka backend sistemu i odgovoran je za:

Prosleđivanje zahteva ka odgovarajućim mikroservisima

Validaciju JWT tokena

Agregaciju podataka iz više servisa (API Composition)

5. Frontend aplikacija

Frontend aplikacija je realizovana korišćenjem Next.js framework-a i omogućava:

Registraciju i prijavu korisnika

Pregled profila pružalaca usluga

Prikaz lokacija na mapi

Pregled i zakazivanje termina

Prikaz QR koda zakazanog termina

Frontend komunicira isključivo sa API Gateway-em.

6. Baze podataka

Svaki mikroservis poseduje sopstvenu bazu podataka u skladu sa principima mikroservisne arhitekture:

Servis	Tip baze
Auth Service	PostgreSQL
User Service	PostgreSQL
Appointment Service	PostgreSQL
Notification Service	Redis / MongoDB

7. Bezbednost

JWT autentifikacija

Role-based autorizacija

Validacija zahteva na nivou API Gateway-a

Zaštita privatnih endpoint-a


9. Zaključak

Implementirani sistem demonstrira primenu mikroservisne arhitekture u realnom problemu zakazivanja termina. Korišćenjem Rust programskog jezika, asinhrone komunikacije, nezavisnih baza podataka i API Gateway-a, sistem je skalabilan, modularan i spreman za dalji razvoj.
