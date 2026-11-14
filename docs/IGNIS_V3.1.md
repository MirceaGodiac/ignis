# IGNIS — Studiul complet și planul unic de execuție

> **Statut (12 noiembrie 2026): document istoric.** Rămâne referință pentru ecuații, surse și infrastructura inițială. Pentru runtime (Codex App Server), WSP literal (fără „NASA training înainte de WSP”), ownership și contracte, autoritatea este `docs/design/IGNIS_MASTER_V2.md` și `docs/implementation/`.

**V3.1 · 7 noiembrie 2026 · plan integrat, fără alocare pe persoane · plafon total de 100 USD**

Acest fișier este ediția autonomă pentru lectură și continuare într-un alt chat. Deciziile actuale sunt separate de variantele istorice; matematica, sursele și limitele sunt incluse în același document. Pentru starea proiectului și regulile de prioritate începeți cu primul capitol. Pentru continuarea implementării folosiți ADR-ul, contractele tehnice și calendarul de execuție.

## Cuprins

- [IGNIS — Studiu integral, specificație științifică și plan de execuție](#chapter-00)
- [1. Cum am ajuns aici și ce decizii sunt abandonate](#chapter-01)
- [2. Un singur produs, trei experți și un circuit de verificare](#chapter-02)
- [3. Inventarul NASA și limitele fiecărei surse](#chapter-03)
- [4. Prelucrare, normalizare și prevenirea informației din viitor](#chapter-04)
- [5. Fundamentul fizic și notația comună](#chapter-05)
- [6. FLEX — model dinamic fizico-neuronal și evenimente](#chapter-06)
- [7. Likelihood de observație coerent pe timpi neregulați](#chapter-07)
- [8. ACME — evenimente dependente de configurație și protocol](#chapter-08)
- [9. BASS — metrologie și operator neuronal al frontului vizibil](#chapter-09)
- [10. Inferența din prefix și acceleratoarele neuronale](#chapter-10)
- [11. Decision Intelligence, active observation și Jev](#chapter-11)
- [12. Campania de training sub plafonul de 100 USD](#chapter-12)
- [13. Verificare: patru întrebări diferite](#chapter-13)
- [14. Stack-ul performant și granițele lui](#chapter-14)
- [15. Bugetul canonic de 100 USD și disciplina Runpod](#chapter-15)
- [16. Execuție paralelă pe module și dependențe, fără împărțire pe persoane](#chapter-16)
- [17. Calendar actualizat și drumul critic](#chapter-17)
- [18. Scientific Intelligence: de la întrebare la dovadă și tool](#chapter-18)
- [19. Dashboardul final și demonstrația](#chapter-19)
- [20. Contracte API, artefacte și reproductibilitate](#chapter-20)
- [21. Literatura: ce am luat în considerare, ce adoptăm și ce nu](#chapter-21)
- [22. Decizia arhitecturală finală și instrucțiuni pentru continuare](#chapter-22)
- [23. Configurațiile inițiale complete și politica de suport](#chapter-23)
- [24. Brief-uri tehnice executabile pe module, fără alocare personală](#chapter-24)
- [25. Proveniența acestei consolidări și limitele de verificare](#chapter-25)
- [26. Verificări sintetice executabile](#chapter-26)
- [Registru complet de surse și proveniență](#chapter-27)
- [28. Receipt de acces la metadate publice](#chapter-28)

---

<a id="chapter-00"></a>

# IGNIS — Studiu integral, specificație științifică și plan de execuție

**Versiune canonică: V3.1 / buget total de 100 USD.**  
**Data reviziei operaționale: 7 noiembrie 2026; cercetarea anterioară și sursele își păstrează datele documentate.**  
**Orizont bibliografic cerut inițial: 28 octombrie 2026.**  
**Echipă: patru persoane, fără atribuirea nominală a fluxurilor în acest plan.**  
**Termen folosit pentru planificare: 14–15 noiembrie 2026, furnizat de echipă.**

## Rezumat executiv

Ignis este un laborator interactiv de cercetare a combustiei în microgravitație. Pornește de la experimente și publicații NASA, le face căutabile și comparabile, apoi permite analiza cantitativă în domeniile unde există măsurători suficiente: dinamica picăturilor FLEX, evenimentele flăcărilor gazoase ACME și geometria fronturilor vizibile BASS. Utilizatorul poate da modelului numai începutul unei înregistrări, poate cere o prognoză cu incertitudine, poate inspecta încă o observație aleasă de sistem și poate compara rezultatul cu continuarea reală, ținută inițial ascunsă.

**Contribuția urmărită nu este numărul rețelelor neuronale, ci circuitul complet:**

> Dovezi → măsurare calibrată → predicție din prefix → alegerea unei observații utile → actualizare probabilistică → predicție înghețată → verificare pe continuarea reală → raport reproductibil.

Produsul central rămâne dashboardul **Explore / Compare / Experiment Lab**, nu un chatbot separat și nici un simulator universal de incendii. Motorul de cercetare poate acoperi multe familii NASA; fiecare predictor are un domeniu fizic precis și separat. Nu aplicăm un model de picături materialelor solide numai pentru că ambele implică foc.

Arhitectura selectată păstrează ambiția V2: trei experți, metrologie video, inferență probabilistică, selecție de observații și proveniență. O organizează însă pentru un **plafon incremental total de 100 USD**. Nu mai presupune un RTX PRO 6000 de 96 GB rezervat permanent, patru fine-tuning-uri SAM sau toate combinațiile de folds și acceleratoare. Modelele fizice și evaluarea sunt prioritare; fine-tuning-ul vizual și rețelele care accelerează inferența se activează când există un beneficiu demonstrabil pe dezvoltare.

Implementarea este realizată de o echipă de patru cu agenți frontier, prin fluxuri tehnice independente, fără o atribuire nominală prestabilită. Planul permite paralelizare amplă și nu folosește durata unei implementări manuale tradiționale ca argument pentru limitarea produsului. În schimb, păstrează verificarea matematică, identitatea datelor, separarea train/test și controlul costului. Absența unei erori de programare nu garantează că datele identifică o mărime fizică sau că un posterior converge bine.

## 0.1. Contextul care trebuie cunoscut într-un chat nou

| Aspect | Starea care controlează proiectul |
|---|---|
| Scop | Un instrument NASA research util, vizual convingător și verificabil, cu diferențiere la hackathon |
| Brief | Dashboard AI care sintetizează, ordonează după relevanță și interpretează cercetările despre combustie în microgravitație |
| Ambiție | Mai mult decât RAG, clasificatoare fără context sau animații decorative |
| Buget | **100 USD total** pentru infrastructură și servicii suplimentare; abonamentele AI deja existente sunt separate |
| Echipă | Patru persoane; planul definește module, interfețe și criterii de acceptare, nu responsabilități individuale |
| Implementare | Intens asistată de LLM-uri frontier, numite de echipă Opus/Astra/Sol; numele nu certifică aici versiuni sau disponibilitate |
| Predictori vizați | FLEX, ACME normal/inverse, BASS într-o cohortă coerentă |
| CPU/GPU | RTX 3090 24 GB în ferestre de calcul; RTX 4090 burst opțional; gazda CPU folosită pentru fitting Bayesian dacă pilotul o justifică |
| Stack | Rust/Axum/Tokio/Rayon; JAX/NumPyro/Equinox; PyTorch/SAM2; SQLite/Parquet; React/TypeScript/uPlot |
| Decision models | Selecție numerică de observații; Jev numai router semantic opțional, fără training propriu |
| Regula de publicare | Se activează numai afirmațiile susținute de măsurare și evaluare; un modul de dovezi rămâne util fără predictor calificat |
| Stare demonstrată | Cercetare, specificații, estimări și teste sintetice; nu există în acest raport un model NASA antrenat/validat sau un deployment verificat |
| Permisiuni | Documentul nu autorizează cheltuieli, crearea de poduri, schimbări în producție sau publicarea de date fără verificarea drepturilor |

## 0.2. Ce este acest document și ce nu este

Acesta este documentul autonom de referință. Include deciziile finale, rațiunea lor, alternativele, matematica implementabilă, datele necesare, protocolul de evaluare, literatura discutată, bugetul, organizarea echipei, contractele software și limitele cunoscute. Pentru a continua într-un alt chat, nu este necesar să fie încărcate din nou toate discuțiile anterioare.

Documentul nu este un jurnal de antrenare și nu conține rezultate predictive fabricate. Inventarele NASA sunt identificate drept **moștenite din auditul anterior**. Referințele bibliografice au niveluri diferite de acces; o lucrare menționată în conversație nu este promovată automat la „citită integral și verificată”. Tarifele cloud sunt ipoteze datate de bugetare, nu rezervări sau oferte garantate. Testele numerice verifică implementări sintetice ale unor identități; nu validează combustia reală.

## 0.3. Ordinea de prioritate a deciziilor

În caz de conflict între artefactele mai vechi, se aplică această ordine:

1. Cerințele actuale explicite ale echipei și regulile oficiale ale competiției, după verificare.
2. **Acest document V3.1**, inclusiv bugetul, calendarul și protocolul de evaluare revizuit.
3. Specificațiile V2, numai în măsura în care nu sunt înlocuite explicit aici.
4. Dosarul V1 și variantele exploratorii, ca istoric și surse de argumente, nu ca instrucțiuni de execuție actuale.

Nu reinstituiți implicit bugetele de 750–950 USD, cele 292 ore de RTX PRO sau obligativitatea tuturor rețelelor de inferență. Nu înlocuiți însă probabilitățile și testele cu rezultate aproximative fără etichetare doar ca să respectați bugetul.

## 0.4. Convenții de proveniență

Fiecare afirmație importantă aparține unei clase:

- **Furnizat de echipă:** preferințe, numărul de participanți, buget, termen, brief și autorizația relatată pentru dezvoltare anticipată.
- **Audit anterior:** constatare din fișierele cercetării V1/V2; nu presupune o nouă descărcare în această consolidare.
- **Referință metodologică:** sursă a unei metode; rezultatele ei nu sunt rezultate Ignis.
- **Pistă neconfirmată:** identificator/titlu de verificat înainte să susțină o alegere decisivă.
- **Propunere Ignis:** ecuație, parametrizare, configurație, prag sau interfață pe care o adoptăm și trebuie să o testăm.
- **Estimare:** timp, cost sau volum de lucru nemăsurat încă pe hardware-ul țintă.
- **Verificare sintetică:** test matematic executabil; separată de validarea experimentală.

Sursele sunt referite prin identificatori stabili, de exemplu [L22](#source-l22) sau [R01](#source-r01), rezolvați în registrul complet de la final. Nu este nevoie de acces la codurile de citare ale unui chat anterior.

---

<a id="chapter-01"></a>

# 1. Cum am ajuns aici și ce decizii sunt abandonate

## 1.1. Evoluția arhitecturii

Prima direcție a fost un motor evidence-first: surse NASA, extragere de tabele și protocoale, căutare hibridă, ranking, citări exacte și explicații. Era util, dar insuficient diferențiat de un dashboard RAG.

A doua direcție a urmărit hărți de flammability, limite de menținere/stingere și regresii fizice. Problema relevantă nu era implementarea unui GP, ci numărul de încercări independente și comparabilitatea protocoalelor.

A treia direcție a propus video → segmentare → măsurători → modele. A rămas valoroasă, dar cu condiția de a separa pixeli de metri, front luminos de piroliză și intensitate de temperatură.

Varianta „physics-grounded world model” propunea simulator de combustie, traiectorii sintetice, FNO/TFNO și adaptare la NASA. Nu a fost selectată ca nucleu: nu era verificat un caz numeric NASA-matched cu câmpuri, condiții la limită și validare independentă. Un surrogate care reproduce CFD nu dovedește că CFD reproduce experimentul.

V1 a convergent pe un model Bayesian comun de diametru și sfârșit al fazei de flacără pentru FLEX metanol, încorporat într-un dashboard general. Alegerea s-a bazat pe istorii măsurate disponibile, nu pe ideea că NASA ar cere numai picături.

Echipa a cerut apoi o ambiție mai mare și utilizarea agenților frontier. V2 a extins produsul cu ACME, BASS, reziduuri neuronale, operator geometric, inferență amortizată corectată și selecție activă de observații. A fost discutat un RTX PRO 6000 cu 96 GB, dar acesta era o opțiune de calcul, nu o cerință matematică.

O primă organizare de echipă a bugetat un pod puternic păstrat aproape permanent. Echipa a respins costul și a fixat ulterior **100 USD**. Această constrângere schimbă campania și persistența infrastructurii, nu justifică schimbarea ascunsă a țintelor științifice.

## 1.2. Registru de decizii înlocuite

| Variantă veche | Decizia actuală | Motiv |
|---|---|---|
| Numai FLEX ca centru al întregului produs | Dashboard general, trei experți țintiți | Domeniul produsului este mai larg decât domeniul unui predictor |
| FastAPI ca backend principal | Rust/Axum; Python/JAX rămâne worker științific | Performanță și limite explicite, fără rescrierea NUTS |
| RTX PRO 6000 permanent | 3090 24 GB în ferestre, burst 4090 opțional | Cost per rezultat, nu VRAM ca scop |
| 500 GB volum permanent | Working set de 100 GB + scratch, backup local | Selectăm activele necesare și decodăm incremental |
| 292 ore la 2,49 USD/h | Maximum 120 ore 3090 +8 ore 4090 în planul de bază | Încadrare în 100 USD |
| Trei folds pentru toate modelele și acceleratoarele | Split grupat train/validation/test, cu extensii numai dacă rămâne buget | Economie declarată; nu pretindem aceeași robustețe între splituri |
| Fine-tuning SAM obligatoriu | SAM pretrained, adaptare condiționată de eroare metrologică | Nu plătim o adaptare fără beneficiu |
| Patru flows obligatorii | Calea corectă fără flow întâi; accelerăm familia cu latență relevantă | Inferența corectă nu depinde de accelerator |
| Student-t innovations pe o anumită cadență | Proces Student-t cu margini coerente pe timpi neregulați | Inserarea unui timp nu schimbă arbitrar legea erorii |
| Decision Transformer / agent RL pentru decizie | VOI/Bayes risk numeric; Jev opțional pentru rutare | Nu avem traiectorii decizionale independente care să justifice RL |
| „Toate rezultatele sunt valide fiindcă agenții nu greșesc” | Teste, referințe independente și holdout | Corectitudinea codului nu dovedește identifiabilitatea fizicii |

## 1.3. Brief-ul competiției și afirmațiile neconfirmate

Brief-ul furnizat de echipă cere un dashboard interactiv AI care sintetizează, ordonează și interpretează cercetările despre combustie în microgravitație, în sprijinul explorării umane. Predicția temporală, comparațiile și selecția observațiilor sunt contribuțiile noastre, nu obligații NASA deduse fără text oficial. [C01](#source-c01)

În conversație au existat afirmații succesive despre publicarea descrierilor la 28 octombrie și a resurselor la 13 noiembrie. **Dosarele recuperate nu conțin o confirmare oficială completă a acestei separări.** Nu tratăm acele date ca fapt verificat. Un eșec de acces al unui instrument nu dovedește că pagina nu este publicată și nici că este necesară autentificarea.

Fluxul de conformitate și integrare păstrează un `requirements_receipt` cu ediția, textul exact, URL, data verificării, formatul de predare, regulile de lucru anticipat, drepturile și eventualele criterii de jurizare. Termenul de 14–15 noiembrie rămâne reperul furnizat de echipă până la reconcilierea oficială. [C01–C04]

„Super intelligence” nu se traduce aici prin construirea AGI, printr-un vendor obligatoriu sau printr-un model care poate inventa măsurători. Îl operacionalizăm printr-un workflow observabil: înțelege întrebarea, găsește dovezi, verifică compatibilitatea, execută calcule permise și explică rezultatele cu limite.

## 1.4. Concurența și diferențierea onestă

**MicroFire Atlas** are un predictor BASS real, delimitarea țintei, verificări de domeniu, comparații și evaluare grupată. În snapshot-ul inspectat, SIBAL folosește grupuri inferate din identificatori cu zi; PMMA este grupat pe run când sesiunea nu este disponibilă. Nu spunem că toate sesiunile fizice sunt cunoscute. Intervalele din codul inspectat includ bootstrap pe înregistrări, lucru diferit de un bootstrap complet pe sesiuni. Nu am reprodus independent performanța lor în această consolidare. [K01–K02]

**Fotia** are deja mai multe modele: clasificări, regresii, random forest pentru ACME și regresii pentru K/diametru final FLEX. README-ul mai vechi nu trebuie folosit pentru a ignora codul existent. Evaluarea random KFold/StratifiedKFold din modulele inspectate justifică o comparație cu splituri pe sesiuni, dar nu dovedește că toate rezultatele autorilor sunt greșite. Loaderul ACME separă deja variabilele post-test; nu inventăm o critică de leakage pentru a ne favoriza. [K03–K06]

FLARENEXUS, ZeroG Ignis și proiectul Mehedi752 au funcții de dashboard și analiză, cu niveluri diferite de date reale, reguli și modele. Starea lor este cea a snapshot-urilor inspectate anterior, nu o certificare a ultimei versiuni sau a codului privat. [K07–K09]

| Dimensiune | Standard minim de atins | Diferențiere vizată de Ignis |
|---|---|---|
| Dovezi | Citare exactă, scope, proveniență | Legătură directă între sursă, observație, model și rezultat reproductibil |
| ML | Baseline-uri și verificări de domeniu | Forecast din prefix și evaluarea continuării ascunse |
| Multimodel | Rutare corectă pe regimuri | Contract numeric comun, nu model fizic universal |
| Incertitudine | Nu prezentăm un scor ca certitudine | Separăm calibrare, observație, model, Monte Carlo și lipsa suportului |
| Decizie | Nu inventăm recomandări de siguranță | Alegem o observație și testăm dacă ea ajută efectiv predicția |
| UX | Produs funcțional și lizibil | Video real + măsurări + forecast + reveal, într-un singur flux |

Nu pretindem să inventăm Bayesian joint modeling, FNO, survival analysis sau value of information. Contribuția este un instrument și un benchmark reproductibil pe date NASA. Complexitatea matematică nu garantează superioritatea față de o regresie simplă; aceasta trebuie demonstrată la același input și pe aceleași grupuri.

---

<a id="chapter-02"></a>

# 2. Un singur produs, trei experți și un circuit de verificare

## 2.1. Întrebările la care răspunde sistemul

**Ce știm?** Motorul de dovezi rezumă și ordonează constatări NASA relevante.

**Ce se poate întâmpla în acest experiment?** Un expert calificat produce o distribuție predictivă asupra unei mărimi definite, folosind numai datele admise.

**Ce merită inspectat în continuare?** Motorul decizional estimează valoarea unei observații suplimentare dintr-un set permis și la un cost declarat.

**A ajutat?** Evaluatorul deschide continuarea reală după înghețarea predicției și calculează scoruri comparabile cu baseline-urile.

## 2.2. Domeniile experților

| Expert | Intrări admise | Ieșire | Ce nu estimează |
|---|---|---|---|
| FLEX | Metanol, condiții reconciliate, diametre în prefix, fază stabilită și suport admis | Diametrul lichidului în timp; distribuția sfârșitului fazei observate/disruption | Masă de metanol pur, câmpuri de temperatură/specii, incendiu de material solid |
| ACME | Configurație normal/inverse separată, condiții inițiale, protocol și informația că faza este încă activă | Timp/cauză de final sub politica experimentală observată | Viața naturală fără intervenție dacă aceasta nu este identificată |
| BASS | Un material/geometrie/protocol calificat, contur calibrat și controale cunoscute | Continuarea frontului vizibil definit | Piroliză, temperatură, certificare de material, generalizare la orice suprafață |

Un experiment din altă familie poate avea documente, grafice și comparații utile fără un predictor dedicat. „Predictor indisponibil” nu înseamnă că platforma nu poate trata experimentul.

## 2.3. Componentele

```text
NASA: publicații + tabele + loguri + video
                 |
                 v
Surse imuabile / identitate / unități / calibrare / rights
                 |
                 v
Registru de experimente și observații + corpus de dovezi
          |                       |
          v                       v
Retrieval / comparabilitate    Reader strict de prefix
          |                       |
          |            +----------+----------+
          |            |          |          |
          |          FLEX       ACME        BASS
          |            +----------+----------+
          |                       |
          |        Condiționare + diagnostice + suport
          |                       |
          |             Artefact numeric imuabil
          |                       |
          |           Selecție de observații / VOI
          |                       |
          +------------+----------+
                       v
          Explore / Compare / Experiment Lab
                       |
                       v
          Reveal controlat / evaluare / research brief
```

Fluxul de date de training și cel de inferență sunt separate. Un parser nu poate publica automat un checkpoint. Un LLM nu poate ocoli lipsa calibrării. Un rezultat numeric se identifică prin hash-ul exact al inputurilor și al versiunilor, nu prin textul pe care îl produce explicația.

## 2.4. Offline, online și opțional

**Offline:** ingestie, audit, calibrare, extragere video, fit de model, selecție de arhitectură, generare de sarcini statistice, training de accelerator, evaluare finală și pregătirea bundle-ului.

**Online:** verificare de domeniu, citirea prefixului, condiționare locală, forecast, calcul de utilitate, căutare și explicație. MCMC global nu se execută la mișcarea sliderului. O rafinare locală dificilă are un job bounded și stare vizibilă.

**Opțional după măsurare:** fine-tuning SAM2, flow-uri pentru toate familiile, GPU serving, Jev, ANN, custom CUDA. Absența acestor optimizări nu schimbă ținta matematică și nu elimină analiza de bază.

## 2.5. De ce nu alegem un backbone universal

Datele nu au aceleași intrări, ieșiri sau stări observabile. Un model comun ar putea învăța identitatea studiului sau textul rezultatului publicat, fără să învețe o relație fizică transferabilă. Punem în comun scheme, cod de inferență, verificări și UI. Punerea în comun a parametrilor fizici cere un experiment explicit de transfer și baseline separat.

Trei experți nu sunt un mixture-of-experts generalist care inventează ieșiri oriunde. Routerul selectează o definiție de problemă. Gate-ul verifică dacă acea problemă este susținută. Dacă niciun expert nu se aplică, răspunsul rămâne evidence-based fără număr predictiv fabricat.

## 2.6. Modulele software și responsabilitățile lor

| Modul | Deține | Nu poate face |
|---|---|---|
| `sources` | URL, versiuni, hash, rights, raw bytes | Modifica originalul ca să ascundă o corecție |
| `normalize` | Unități și transformări documentate | Inventar de gaze lipsă sau valori tipice introduse ca măsurate |
| `curate` | Identitate, cohortă, fază, endpoint review | Atribuire prin vot LLM în absența dovezii |
| `science` | Ecuații, likelihood, posterior, baseline | Acces la continuarea unui test în timpul forecast-ului |
| `vision` | Măști și contururi cu versiune/calibrare | Folosirea viitorului în calea cauzală |
| `runtime` | API, validare, cache, job limits | Publicare automată a unui model neverificat |
| `evidence` | Findings, retrieval, comparabilitate | Transformarea oricărui citat într-o afirmație cauzală |
| `decision` | Acțiuni permise, utilitate, cost | Comenzi asupra unui experiment fizic real |
| `evaluation` | Splituri, scorer, reveal, raport | Retuning pe test și păstrarea aceleiași etichete de independență |
| `web` | Vizualizare și interacțiune | Înlocuirea unui eșec numeric cu o curbă decorativă |

## 2.7. Datele necesare pentru un rezultat valid

O predicție cere mai mult decât un fișier CSV. Trebuie să știm ce run reprezintă, ce instrument a produs valoarea, unitatea, originea timpului, ce control era activ, dacă faza modelată era stabilită, ce înseamnă eventualul endpoint și ce observații au fost disponibile la cutoff. Lipsa unuia dintre aceste elemente poate permite explorare descriptivă, dar nu aceeași precizie a prognozei.

---

<a id="chapter-03"></a>

# 3. Inventarul NASA și limitele fiecărei surse

## 3.1. Ce înseamnă un număr din acest inventar

Numerele de mai jos sunt constatări **raportate în auditul anterior**, păstrate pentru continuitate. Nu sunt o nouă descărcare completă, nu certifică integritatea tuturor activelor în versiunea actuală și nu reprezintă automat numărul de experimente independente de antrenare. La implementare se regenerează manifestul, se compară hash-urile și se publică un flow table al includerilor/excluderilor.

Diferențiem fișiere, cadre, măsurători, segmente de control, arderi, specimene reutilizate, sesiuni și condiții independente. O sută de cadre ale aceleiași flăcări nu reprezintă o sută de incendii independente. O publicație care reproduce un grafic din FLEX nu adaugă un nou experiment.

| Sursă | Inventar anterior relevant | Rol curent | Blocaj de calificare |
|---|---|---|---|
| FLEX PSI-69 [D01](#source-d01) | 274 rânduri sumar; 260 istorii de diametru listate; 152 istorii de metanol raportate ca descărcate | Expert dinamic principal | Identitate, fază, calibrare, endpoint, grupare |
| BASS-II PSI-25 [D02](#source-d02) | 129 rânduri master; 107 etichete brute de probă; 296 MP4 listate | Metrologie și expert de front | Mapping video/run/specimen și calibrare |
| BASS PSI-26 [D03](#source-d03) | 122 rânduri; protocol înrudit, nu identic | Dovezi și subset separat | Nu combinăm automat cu BASS-II |
| ACME CFI-G PSI-159 [D04](#source-d04) | 179 normal +154 inverse ne-goale | Expert de evenimente | Timp, intervenții, aprindere, rolul compoziției |
| Saffire I/II/III [D05–D07] | 2/9/2 înregistrări de arderi/probe în tabelele inspectate | Metrologie/scară/comparații | Nu rezultă același număr de specimene independente |
| FLEX-2 [D08](#source-d08) | Arhivă mare; opt intrări sumar de familii de combustibil/design | Extensie ulterioară și dovezi | Pipeline diferit, interacțiuni/mișcare/compoziții |
| SPICE [D09](#source-d09) | 469 rânduri, 34 duplicate exacte, 151 perechi dată/aprindere | Geometrie/funingine și dovezi | Mai multe stări de debit per flacără |
| PSI-117 [D10](#source-d10) | Curbe experimentale și simulate asociate figurilor | Verificări matched-case | Proveniență și suprapunere cu FLEX |
| PSI-60/62 [D11–D12] | ZIP-uri mici de metadate în release-urile auditate | Piste de literatură | Nu au stabilit un corpus CFD reutilizabil |
| ACME Flame Design [D13](#source-d13) | 207 normal +120 inverse în tabelele anterioare | Dovezi / posibil expert viitor | Condiții ramped, licență, observabile |
| BRE / S-Flames [D14–D15] | 194 /276 aprinderi însumate din câte 18 zile | Dovezi | Nu există încă registry calificat pe fiecare run |
| PSI-142 [D16](#source-d16) | Curbe/tabele counterflow la 1 g | Referință terestră separată | Nu se antrenează împreună cu ISS fără studiu de transfer |
| CFI / SAME-R [D17–D18] | Tabele folosite de concurenți pentru flăcări reci/detecție | Extensii descriptive | Targeturi și grupuri de verificat independent |
| SoFIE-GEL [D19](#source-d19) | 54 arderi pe 17 sfere reutilizate în descriere | Dovezi și adaptor pregătit | Descărcarea completă pe run nu este stabilită |
| ZARM/FLARE [D20](#source-d20) | 30 fișiere MAT listate, 31 intrări index; ~3,8 GB | Pista termică separată | CC BY-NC raportat, geometrie diferită și un activ lipsă |

## 3.2. FLEX: ce luăm efectiv

Prima cohortă numerică este **metanol liber flotant**, într-o fază observată și calificată după retragerea aprinzătorului. Nu includem heptanul, suportul pe fibră, mișcarea impusă ori picături multiple doar pentru a mări setul.

Auditul anterior a raportat 152 istorii de diametru ale metanolului, 6.055.320 bytes și 85.574 rânduri de măsurători. Dintre identitățile candidate, 147 aveau data/GMT reconciliate, pe 50 date; 117 aveau flag-ul structurat fără fibră, pe 47 date. Narațiunea poate contrazice un flag. Cohorta finală nu este încă stabilită.

Fișierele de lucru sunt: summary-ul original, test sheets, `DropletData`, canale `ColorCamData` și `LLUVData` potrivite, poziția picăturii, documentele de calibrare și câteva clipuri necesare adjudicării. Un fișier de cameră existent nu garantează imagini utilizabile.

`DropletDiameter` este geometria lichidului derivată din imagini, nu temperatura și nici masa de metanol pur. Canalele vizibil și UV urmăresc alte obiecte și pot avea altă origine temporală. Procesele de absorbție/evaporare a apei contează pentru interpretare. [P01–P03]

### Probleme cunoscute de reparat transparent

Antetele `O`, `N`, `CO` au fost reconciliate în audit cu O₂, N₂, CO₂. Rata de ardere are unitate lungime²/timp, nu lungime. Păstrăm eticheta originală și justificarea corecției.

Nu legăm fișierele prin numărul aparent FLEX din nume. Au fost găsite nepotriviri sistematice între numele fișierului și numărul din tabel. Cheia este CIR + dovezi de dată/GMT, păstrând toate aliasurile.

Auditul a semnalat valori nepozitive/nefinite în 11 trasee, duplicate de timestamp în două și goluri peste o secundă în 17. Nu presupunem că un filtru care elimină acestea produce automat date fizic valide. Timpul zero procesat nu este implicit momentul aprinderii. Diametrul poate continua să fie măsurat după sfârșitul flăcării vizibile.

Pentru fiecare prefix se verifică ce era cunoscut atunci. Dacă ulterior apar drift, expansiune, ieșire din câmp sau ruperea picăturii, cazul rămâne în contabilitatea evaluării. Nu se elimină retrospectiv fiindcă forecast-ul era dificil.

## 3.3. ACME: ce trebuie adjudicat

Normal și inverse sunt două configurații fizice distincte. În normal se injectează combustibil în mediu oxidant; în inverse se injectează oxidant într-un mediu combustibil. O coloană numeric similară poate avea alt rol fizic.

Setul de 333 rânduri ne-goale nu este echivalent cu 333 etichete curate. Loaderul concurentului identificase 272 rânduri SE/FT și 61 fără această etichetă; acest număr trebuie reconciliat cu sursa primară și cu criteriul nostru de fază stabilită. [K05](#source-k05)

Păstrăm separat `Hot ignition = yes/no/flash`, durata, categoria brută, comentariul, data/test day, debitul injectat, presiunea și compoziția pre-test. Un `flash` nu intră automat în populația flăcărilor stabilite.

Un exemplu inspectat anterior, 21088 C1, are abrevierea SE și un comentariu despre atingerea limitei temperaturii arzătorului. Aceasta este o contradicție de adjudicat din log/video, nu o etichetă ce se rescrie automat. Dacă intervalul și cauzele posibile sunt cunoscute, se poate folosi un likelihood cu set de cauze; dacă nu, rămâne neeligibil pentru componenta de evenimente.

Condițiile post-test, temperatura maximă a arzătorului, durata finală și eticheta finală nu devin inputuri prospective. Tad și Zst sunt proprietăți calculate ale amestecului, nu două rotițe independente pe care le poate schimba arbitrar UI-ul. TFP rămâne exclus până la definire.

## 3.4. BASS: ținta este o observație geometrică

Cohorta inițială propusă este planar SIBAL, cu geometrie și protocol compatibile, **numai dacă mapping-ul și calibrarea o califică**. Aceasta este o decizie de domeniu, nu o afirmație că toate clipurile necesare au fost deja găsite. Un alt subset planar poate înlocui cohorta prin revizie documentată înainte de scorurile finale; nu alegem subsetul care produce retrospectiv curbele cele mai frumoase.

Pentru fiecare clip avem nevoie de run/specimen/session, timestamp real, dimensiuni și orientare, distorsiune optică, transformarea pixel→metri și istoricul controlurilor. O homografie a suprafeței probei nu elimină eroarea dacă flacăra luminoasă este semnificativ în afara acelui plan. În acel caz includem o limită geometrică justificată ori nu producem o măsurătoare SI de aceeași precizie.

Un contur leading-luminous este definit înainte de anotare. Îl păstrăm ca atare. Când conturul devine multivaluat, desprins sau ocluzionat, raportăm acel status; un fit circular ori o spline netedă nu trebuie să ascundă schimbarea de topologie.

Planul anterior folosea pentru dimensionare 20–40 clipuri și 800–1.500 cadre de referință. Sunt ținte de lucru, nu sample size verificat. Sub bugetul de 100 USD, selectăm cadre pentru acoperirea run-urilor și situațiilor dificile, nu pentru multiplicarea artificială a numărului de exemple.

### De ce nu folosim direct cele 52 segmente PMMA drept câmpuri de front

Suplimentul studiului Link et al. conține rate scalare de propagare pe 52 segmente cu etichete repetate. Auditul anterior a găsit 19 etichete tip sample/test și o nepotrivire între cm/s și mm/s în jumătățile tabelului. Un diametru de probă este 0,64 cm, nu 0,64 mm. Identitățile și unitățile trebuie reconciliate înainte de un fit numeric. Ratele scalare nu oferă automat contururi spațiale pentru operatorul nostru. [P05](#source-p05)

## 3.5. Restul surselor nu sunt abandonate

Saffire permite explicații despre scară, materiale, ventilație și metrologie. SoFIE permite organizarea dovezilor despre solide și evidențierea unui gol de acces. SPICE permite investigarea funinginii fără a transforma flow-ul măsurat la smoke point într-un predictor cunoscut anterior. FLEX-2 și FLARE pot susține extensii separate după audit.

Nu confundați „nu am localizat public măsurătorile” cu „datele nu există”. Nu confundați „un proiect are multe fișiere” cu „avem multe traiectorii independente calificate”.

## 3.6. Descărcare reproductibilă și working set

Pentru PSI, auditul anterior a documentat metadata endpoint-ul `https://psi.nasa.gov/geode-py/ws/repo/investigations/PSI-69`, listarea read-only `POST /geode-py/ws/files/v2` și download-ul stabil pe investigație/filename/file-version. Acestea sunt piste de reproducere care trebuie verificate înainte de automatizare; nu tratăm un endpoint intern ca API garantat permanent.

Manifestul local conține investigație, versiunea investigației, versiunea fișierului, numele exact, URL stabil, mărime, SHA-256, data preluării și rights. Nu salvăm drept identificator numai un redirect semnat care expiră. O schimbare de versiune produce un nou manifest și nu modifică în tăcere un split înghețat.

Working set-ul cloud este 100 GB: activele curente, măsurători compacte, ponderi și checkpoint-uri. Originalele suplimentare pot rămâne pe calculatoarele echipei cu două copii verificate. Extragem video incremental; nu păstrăm gratuit milioane de PNG-uri. Refacerea din sursa originală trebuie să rămână posibilă.

---

<a id="chapter-04"></a>

# 4. Prelucrare, normalizare și prevenirea informației din viitor

## 4.1. Pipeline-ul unic

```text
bytes originale → hash → parser → câmpuri brute → corecții motivate
→ unități SI → identitate/grupuri → fază/calibrare/endpoint
→ cohortă → split înghețat → normalizare de model → batch-uri
```

Originalul este read-only. O corecție produce un câmp nou cu motiv și sursă. Nu suprascriem `CO` cu `CO2` fără un record al rațiunii. Nu convertim o celulă `-`, `#VALUE!` sau `#DIV/0!` într-un zero măsurat.

## 4.2. Unități și scale

| Original | SI intern | Observație |
|---|---|---|
| mm | ×10⁻³ m | Afișarea în mm este permisă |
| mm²/s | ×10⁻⁶ m²/s | Nu se confundă cu viteza m/s |
| cm/s | ×10⁻² m/s | Direcția are convenție explicită |
| bar | ×10⁵ Pa | Presiune absolută versus relativă de documentat |
| atm standard | ×101325 Pa | Nu confundați cu o valoare exactă a presiunii ISS |
| mmHg convențional | ×133,322387415 Pa | Convenția sursei rămâne în metadata |
| mg/s | ×10⁻⁶ kg/s | Flux injectat versus rată derivată nu sunt interschimbabile |
| °C | +273,15 K | Logaritmii folosesc rapoarte adimensionale |
| % molar | /100 | Nu fracție masică |

Nu normalizăm compoziția gazului la sumă unu fără a explica de ce suma originală diferă. Pentru rotunjiri, incertitudinea și corecția trebuie documentate. Un gaz necunoscut nu este zero. Oxigenul molar și presiunea totală rămân intrări separate; pO₂=χO₂p este o derivare, nu un control independent în plus.

Scalele fizice fixe pot fi declarate înainte de split. Standardizarea statistică, imputarea justificată și selecția caracteristicilor folosesc numai training. Datele de test nu intră în min/max, PCA, selecția de noduri, bin-uri temporale sau alegerea exemplului demonstrativ pe baza erorii.

## 4.3. Identitate și grupare

Un `Run` aparține unei investigații, unui protocol și unei sesiuni; poate avea un specimen reutilizat. Un frame, o valoare într-un CSV, un punct digitizat și o figură publicată care descriu aceeași ardere păstrează același run ID. Suprapunerea nu se rezolvă prin numele diferit al fișierului.

Splitul pentru o afirmație de generalizare la specimene noi nu poate lăsa același specimen reutilizat în train și test, chiar dacă datele calendaristice diferă. Dacă grupurile se unesc într-o componentă foarte mare, raportăm limitarea. Nu o rezolvăm prin split aleatoriu de cadre.

Pentru cohorta mică, procentele sunt orientative. Un punct de pornire este 60% grupuri train, 20% validation și 20% final-test, cu stratificare justificată după metadata cunoscute. Numărul efectiv de evenimente și grupuri poate impune altă repartizare. Regula și seed-ul se îngheață înainte de scoruri, nu se resamplează până apare un test favorabil.

## 4.4. Prefix cauzal

Un `PrefixBundle` este un obiect imuabil. Conține numai:

- condiții și controale cunoscute până la cutoff;
- observații efectiv admise, cu timestamp și calibration ID;
- definiția fazei, dovezi de activitate la cutoff și informația de suport disponibilă atunci;
- versiunea transformărilor și hash-ul inputurilor.

Nu conține rezultatul final, full-history K, diametrul terminal, durata completă, peak temperature, descrierea finalului dintr-o publicație sau o mască corectată folosind un cadru viitor.

**Cauzalitatea se aplică și la metrologie.** O mască dintr-un frame admis care a fost recalculată prin tracking invers folosind o imagine viitoare are informație viitoare, chiar dacă timestamp-ul măștii este vechi. Pentru benchmark, memoria tracker-ului și prompturile au propriul manifest de dependențe.

Același principiu se aplică selecției unei observații ascunse din prefix: acea imagine nu poate fi „ascunsă” în UI dar deja folosită pentru tracking-ul cadrelor admise. Graful de proveniență trebuie să dovedească izolarea.

## 4.5. Timp și eșantionare

Folosim timestamp-uri reale și originea temporală adjudicată, nu numărul de frame împărțit la un FPS presupus. Un video cu variable frame rate are PTS. Offset-urile între canale sunt separate și justificate.

Pentru pilotul FLEX, grila de 0,2 s selectează ultimul punct valid din interval care era disponibil la cutoff, cu timestamp-ul său real. Un interval gol rămâne gol. Nu duplicăm o măsurătoare ca să umplem o matrice. Padding-ul este numai computațional și este mascat înainte de operații care ar produce NaN/Inf.

O verificare de sensibilitate pe dezvoltare compară o cadență mai fină. Modelul de observație V3 este definit pe timpi neregulați, deci subseturile sunt margini coerente; nu pretindem că thinning-ul nu pierde informație.

## 4.6. Endpoint și măsurare

Recordul de endpoint păstrează categoria originală, cauzele admise după adjudicare, ultimul timp confirmat activ, primul timp terminal, timpul de intervenție și starea camerei. Necunoscut nu este o clasă fizică nouă de antrenare.

Datele după ultimul timp confirmat activ nu se folosesc automat în likelihood-ul fazei hot. Ele pot fi utile în viewer și pentru o fază separată, nu pentru a prelungi tacit modelul actual.

Cenzurarea este o observație incompletă, nu un eveniment negativ. Cenzurarea independentă este o ipoteză condiționată pe covariate, nu un fapt garantat de utilizarea cuvântului „censor”. Drift-ul poate influența și pierderea vizibilității, și arderea.

## 4.7. Contract minimal de observație

```json
{
  "run_id": "canonical-id",
  "quantity": "liquid_diameter",
  "time_s": 2.4,
  "raw_value": 2.15,
  "raw_unit": "mm",
  "value_si": 0.00215,
  "unit_si": "m",
  "calibration_id": "calibration-version",
  "quality_flags": [],
  "evidence_type": "researcher_derived_measurement",
  "source_locator": {"asset_sha256": "...", "row": 125},
  "available_at_s": 2.4
}
```

Exemplul este o schemă ilustrativă, nu un rând NASA. Incertitudinea poate fi `null` cu motiv; nu inserăm o deviație standard imaginară. Flag-urile și lipsurile se păstrează în scoruri și în flow table-ul cohortelor.

---

<a id="chapter-05"></a>

# 5. Fundamentul fizic și notația comună

Ecuațiile sunt etichetate **[LEGE]**, **[METODĂ]**, **[APROXIMARE]** sau **[IGNIS]**. O ecuație [IGNIS] este o alegere de modelare care trebuie verificată, nu o lege descoperită a combustiei.

## 5.1. Notație

| Simbol | Sens | Unitate |
|---|---|---|
| i,j,g | Run, observație, grup de sesiune/calibrare | indici |
| t,τ | Timp de la intrarea revizuită în fază; cutoff | s |
| c,u(t) | Condiții normalizate; controluri admise | adimensional / unitatea controlului |
| Θ,b,κ | Parametri globali, efecte locale, calibrare | conform definiției |
| D,a,q | Diametru, rază, q=D² | m, m, m² |
| K | Rată de scădere a diametrului pătrat | m²/s |
| ρ,p,T | Densitate, presiune, temperatură | kg/m³, Pa, K |
| χ,Y | Fracție molară / masică | adimensional |
| λ,S,F | Hazard, supraviețuire de fază, incidență cumulativă | s⁻¹, probabilitate, probabilitate |
| B,U(t) | Timp de ieșire din suport; masă predictivă nerezolvată | s, probabilitate |
| f(s,t),W,L | Poziție front vizibil; lățime și lungime probă | m |
| ν_f | Regularizare cinematică de curbură | m²/s, nu difuzivitate termică măsurată |
| Σ | Matrice de scară Student-t | pătratul unității observației |
| R_B,U_a | Bayes risk; utilitate a achiziției | unitatea loss-ului |

Toți logaritmii unor mărimi fizice sunt logaritmi ai raportului la o unitate/scală declarată. Numere precum D_ref=0,002 m sunt scale de condiționare, nu constante ajustate de NASA.

## 5.2. Conservarea în gaz: contextul pe care nu îl simulăm complet

Un model low-Mach cu densitate variabilă trebuie să respecte [LEGE + APROXIMARE low-Mach]:

\[
\partial_t\rho+\nabla\cdot(\rho\mathbf u)=0,
\]
\[
\partial_t(\rho Y_k)+\nabla\cdot(\rho\mathbf uY_k)
=-\nabla\cdot\mathbf J_k+\dot\omega_k,
\quad\sum_kY_k=1,\quad\sum_k\mathbf J_k=\sum_k\dot\omega_k=0,
\]
\[
\rho D_t\mathbf u=-\nabla\pi+\nabla\cdot\boldsymbol\tau+\rho\mathbf g,
\qquad p_0=\rho R_uT/\overline W.
\]

Cu entalpia de formare inclusă în entalpia amestecului h:

\[
\rho D_t h=\frac{dp_0}{dt}+\nabla\cdot(k\nabla T)
-\nabla\cdot\sum_kh_k\mathbf J_k-\nabla\cdot\mathbf q_{rad}.
\]

Nu adăugăm încă o dată căldura chimică dacă ea este deja inclusă în această convenție. Densitatea variabilă nu presupune div u=0. Transportul, cinetica, radiația, geometria și condițiile inițiale/la limită trebuie să închidă sistemul.

Aceste ecuații explică de ce o filmare sau un diametru nu identifică toate câmpurile. **Ignis nu pretinde că le rezolvă din RGB.** Un solver Cantera/OpenFOAM/FDS existent nu oferă automat un caz calibrat pentru experimentele noastre. [T23–T25]

## 5.3. Derivarea reducerii D²

Pentru lichidul sferic [LEGE]:

\[
m_l=\frac\pi6\rho_lD^3.
\]

Cu q=D² și pierdere netă de masă pozitivă spre exterior:

\[
\dot q=-\frac{4\dot m_{net,out}}{\pi\rho_lD}
-\frac{2q}{3\rho_l}\dot\rho_l.
\]

Sub aproximarea sferică quasi-staționară, proprietăți constante și fără absorbție către lichid [APROXIMARE]:

\[
\dot m_{out}=2\pi D\rho_g\mathcal D\log(1+B_M),
\]
\[
\dot q=-K,\quad
K=\frac{8\rho_g\mathcal D}{\rho_l}\log(1+B_M),
\quad D^2(t)=D_0^2-Kt.
\]

La interfața mobilă, fluxul fiecărei specii este consistent cu masa netă. Pentru raza a(t),
\(\rho_g(u_g-\dot a)Y_k+J_k=\dot m''_k\), iar suma fluxurilor speciilor este fluxul net. Apa care condensează și metanolul care se evaporă au semne opuse. Bilanțul de energie include fluxuri conductive/radiative și cererea de schimbare de fază, fără dublă contabilizare.

De aceea q este geometria lichidului, nu inventarul de metanol. Extrapolarea matematică până la q=0 nu este o predicție verificată a consumării combustibilului. [P01–P03]

## 5.4. Arderea solidelor și limita observabilă

Un model redus noncharring poate folosi conversia α:

\[
\partial_t\alpha=A_p\exp[-E_p/(R_uT_s)](1-\alpha)^n,
\quad\dot\omega_p'''=\rho_{s,0}\partial_t\alpha,
\]
\[
\rho_{s,0}c_s\partial_tT_s=\nabla\cdot(k_s\nabla T_s)-\Delta h_p\dot\omega_p'''.
\]

Fluxul gazos emis și bilanțul de suprafață trebuie legate de pierderea de masă și de încălzire. Char, porozitate, deformare, recesiune și istoria termică cer alte stări. Termenul endotermic deja reprezentat în volum nu se numără din nou ca aceeași pierdere la suprafață.

**Operatorul BASS nu estimează α sau T_s.** El prezice conturul vizibil măsurabil. Acesta este un model cinematic ghidat de geometrie, nu o rezolvare ascunsă a pirolizei.

## 5.5. Adimensionalizare și comparabilitate

Pentru picături, r=q/q₀ și t*=K_ref t/q₀ sunt direct utile. Pentru un front, ξ=s/W și η=f/L. Dacă proprietățile și mișcarea sunt efectiv cunoscute, Re=UD/ν, Pe=UD/𝒟 și Da=t_transport/t_chimie pot orienta comparația. Pentru solide, Fo=α_th t/L² și Bi=hL/k descriu rolul memoriei termice și al transferului.

Nu calculăm aceste numere folosind temperaturi sau difuzivități inventate. Nondimensionalizarea nu face interschimbabile filmul, cilindrul, picătura și arzătorul. Nu impunem global „mai mult airflow = mai mult foc”: quenching, transport de oxigen, pierderi de căldură și blowoff pot produce relații nemonotone, dependente de geometrie și protocol. [P04–P05]

## 5.6. Modelul probabilistic comun

Pentru FLEX și ACME:

\[
p(\Theta,\kappa,\{b_i,x_{0i}\}\mid\mathcal D)
\propto p(\Theta)p(\kappa)\prod_i p(b_i,x_{0i}\mid\Theta)
L_i^{obs}L_i^{event}.
\]

Fiecare eveniment contribuie o dată per run. Fiecare prior de calibrare comună contribuie o dată per grup, iar observațiile folosesc aceeași valoare latentă. Likelihood-urile sunt normalizate; nu adăugăm ponderi arbitrare pentru a face loss-ul de eveniment comparabil cu cel de diametru.

BASS folosește un ansamblu empiric de modele MAP, nu un posterior Bayesian exact asupra tuturor ponderilor. În inferența locală putem defini o distribuție condiționată precisă asupra indicelui de ansamblu și parametrilor locali. Denumirea incertitudinii rămâne însă distinctă.

---

<a id="chapter-06"></a>

# 6. FLEX — model dinamic fizico-neuronal și evenimente

## 6.1. Întrebarea și populația

Întrebarea este: pentru o picătură de metanol liber flotantă, în faza observată deja stabilită, ce putem anticipa despre geometria lichidului și sfârșitul fazei, folosind condițiile și un prefix de diametre? Nu estimăm probabilitatea inițială de aprindere pentru toate picăturile distribuite.

Intrarea în fază este adjudicată din protocol și dintr-o regulă past-only. Nu alegem retrospectiv începutul „porțiunii care se potrivește cel mai bine”. Cohorta și regulile de suport sunt înghețate înainte de test.

## 6.2. Caracteristici și parametrizare

Fixăm [IGNIS]: D_ref=0,002 m, p_ref=100000 Pa, K_ref=10⁻⁶ m²/s și t_ref=1 s.

\[
c=\left(\frac{\chi_{O_2}-.21}{.05},\frac{\chi_{CO_2}}{.20},
\frac{\chi_{He}}{.20},\log\frac p{p_{ref}},\log\frac{D_0}{D_{ref}}\right),
\]
\[
z(c)=(1,c_1,c_2,c_3,c_4,c_5,c_1^2,c_1c_2,c_1c_4).
\]

D₀ este o stare latentă pozitivă la intrarea în fază. Coloanele fără variație sau fără date admise se elimină printr-o regulă training-only, iar numărul de parametri se actualizează. Regularizarea nu identifică separat presiunea și un diluant dacă designul experimental le confundă.

Cu q=D², r=q/q₀ și w=1−r, definim o rețea 6→16→1:

\[
h_\phi(c,w)=\tanh\{W_2\tanh[W_1(c,w)+a_1]+a_2\}.
\]

Ea are 6·16+16+16+1=129 parametri. Rezidualul este

\[
R_\phi(c,w)=\tfrac12 w\,[h_\phi(c,w)-h_\phi(0,w)].
\]

Acesta dispare la intrare și la condiția de referință. Pentru 0≤w≤1, |R|≤w≤1. Limita este o decizie de regularizare a modelului, nu o lege fizică.

## 6.3. Dinamica

\[
\boxed{\frac{dq_i}{dt}=-K_{ref}\exp\left[
z(c_i)^\top\beta+\theta_1w_i+\theta_2w_i^2+\theta_3w_i^3
+b_{0i}+b_{1i}w_i+R_\phi(c_i,w_i)\right].}
\]

Efectele locale sunt
\(b_i\sim N(0,\Sigma_b)\), cu \(\Sigma_b=\operatorname{diag}(\sigma_b)\Omega_b\operatorname{diag}(\sigma_b)\).

Exponențiala impune o rată pozitivă de scădere a q. Ea nu impune ca fiecare măsurătoare zgomotoasă să scadă. Modelul este admis numai într-o fază în care această presupunere latentă este justificată. O expansiune fizică ulterioară este un eșec de suport evaluat, nu o dovadă care poate fi ștersă.

La aceeași stare și aceiași parametri non-neuronali, rezidualul multiplică rata cu o valoare între exp(−w) și exp(w). Fără rezidual, recuperăm V1. Fără curburi și cu rată constantă, recuperăm baseline-ul D². Modelul neuronal trebuie să îmbunătățească ceva verificabil față de aceste cazuri, nu doar să aibă un nume mai modern. [L21](#source-l21)

## 6.4. Integrare separabilă și gradient

Pentru parametri și condiții fixe în intervalul admis:

\[
t_i(r)=A_i\int_r^1\exp[-g(1-u)-b_{1i}(1-u)-R_\phi(c_i,1-u)]\,du,
\quad A_i=\frac{q_{0i}}{K_{ref}}e^{-z(c_i)^\top\beta-b_{0i}}.
\]

Integrandul este pozitiv, iar t(r) este strict descrescător în r. Inversarea este unică pe [r_min,1]. Calculăm o integrală cumulativă per draw/run, apoi folosim bracket și rafinare locală. Nu repetăm o cuadratură completă în fiecare pas de bisection pentru fiecare observație.

Pentru F(r,ϑ)=t(r;ϑ)−t_obs=0:

\[
\frac{\partial r}{\partial\vartheta}
=-\frac{\partial_\vartheta F}{\partial_rF},
\qquad\partial_rF=-A_i\exp[-g(1-r)-b_{1i}(1-r)-R(c_i,1-r)]<0.
\]

Se folosește diferențiere implicită testată față de finite differences, nu diferențierea deciziilor booleene ale bisection. Tabelele care depind de parametri se recalculează când draw-ul se schimbă. Caching-ul nu este o permisiune de a reutiliza o traiectorie pentru alți parametri.

Un solver de referință float64 cu cuadratură de precizie mai mare verifică implementarea rapidă. Toleranțele se stabilesc în unități fizice și comparativ cu incertitudinea de măsurare, nu doar în număr arbitrar de iterații.

## 6.5. Endpoint-uri și hazarduri

H înseamnă sfârșit revizuit al fazei hot observate. Se păstrează subetichetele sursei extinction/completion, fără a afirma că toate reacțiile s-au oprit. R înseamnă disruption care invalidează modelul picăturii unice. FOV/camera loss este un proces de observare, nu H.

\[
\lambda_H(t)=t_{ref}^{-1}\exp[\gamma_0+\gamma^Tc+a_1w(t)+a_2w(t)^2
+\rho_0b_0+\rho_1b_1],
\]
\[
\lambda_R(t)=t_{ref}^{-1}\exp\zeta.
\]

Termenii comuni b leagă evoluția măsurată de distribuția evenimentului. Asocierea nu identifică un mecanism chimic cauzal. Hazardul de disruption cu intercept este o aproximare parsimonioasă; nu demonstrează că disruption este independent de condiții în natură.

\[
S(t)=\exp[-\int_0^t(\lambda_H+\lambda_R)du],
\qquad F_k(t)=\int_0^tS(u)\lambda_k(u)du.
\]

Pe domeniul modelat, S+F_H+F_R=1. O observație exactă de cauză k la T contribuie S(T)λ_k(T). Un interval de cauză cunoscută (L,R_e] contribuie

\[
L^{event}=\int_L^{R_e} S(u)\lambda_k(u)du.
\]

**S(L)−S(R_e) este probabilitatea tuturor cauzelor**, nu doar a uneia. Pentru o cauză necunoscută, însumăm numai cauzele permisibile din recordul adjudicat. O observație confirmată activă până la C contribuie S(C) numai sub modelul de observare/cenzurare declarat.

## 6.6. Ieșirea din suport și probabilitatea nerezolvată

Definim r_min=q_min/q₀>0 din limita măsurării/modelului și B=min(t(r_min),T_valid), cu restricțiile suplimentare declarate. B poate diferi între draws. Nu extrapolăm în afara lui.

Pentru un interval de eveniment despre care review-ul stabilește că evenimentul a precedat ieșirea din suport, likelihood-ul este

\[
\mathbf1_{B>L}\int_L^{\min(R_e,B)}S(u)\lambda_k(u)du.
\]

Nu inventăm această ordine doar fiindcă ne convine numeric. Dacă sursa nu o stabilește, endpoint-ul nu poate fi reparat prin truncarea lui pe fiecare draw. Un datum nu devine event pentru un draw și censor pentru altul. Draw-urile incompatibile cu informația păstrată despre activitate/suport au likelihood zero.

La cutoff τ, distribuția este condiționată pe activitate și suport până la τ. Pentru t≥τ:

\[
A(t)=E[\mathbf1_{t<B}S(t)/S(\tau)],
\quad U(t)=E[\mathbf1_{t\ge B}S(B)/S(\tau)],
\]
\[
F_k^\tau(t)=E\left[\int_\tau^{\min(t,B)}\frac{S(u)}{S(\tau)}\lambda_k(u)du\right].
\]

A+U+F_H^τ+F_R^τ=1. U este lipsa unei prognoze rezolvate după limita modelului, nu o cauză fizică de stingere. Nu eliminăm draws din suport pentru a renormaliza și afișa o certitudine falsă.

Diametrul condiționat pe fază încă activă și suport folosește ponderi proporționale cu 1_{t<B}S(t)/S(τ). Banda acestei cantități nu este o bandă necondiționată pentru toate experimentele. Distribuția viitoarei măsurători include în plus calibrarea și erorile observaționale condiționate pe prefix.

## 6.7. Priori, număr de parametri și identificabilitate

Inventarul nominal: 9 β +3 θ +3 covariance b +10 hazard H +1 disruption +3 kernel observație +129 NN = **158 populație**. Fiecare run adaugă b₀,b₁ și log D₀, adică 3N; nuisances de calibrare justificate se numără separat.

Punct de pornire pentru priori [IGNIS]: β₀~N(log0,5,1²); efecte liniare N(0,0,5²), interacțiuni N(0,0,25²); θ~N(0,0,35²); scale b half-normal(0,5), Ω_b~LKJ(2). Hazard intercept H~N(log0,1,1,5²), restul efectelor N(0,0,5²); ζ~N(log0,01,1,5²). Acestea sunt ipoteze pe scale normalizate, nu măsurători NASA.

W₁ are prior Gaussian cu scală 1/√6 și biases mici; W₂ are scală inițială 0,25/√16 și bias mic. Inițializăm departe de un punct complet zero care poate bloca gradientul rezidualului diferențiat. Simetriile ponderilor pot afecta diagnosticele MCMC; urmărim și convergența funcției predictive, fără a ascunde instabilitatea sub această explicație.

Pentru D₀ și scalele erorii folosim priori log-normali largi cu unități explicite, verificați prin prior predictive. Un exemplu inițial log(D₀/1 m)~N(log0,003,0,7²) nu trebuie centrat îngust pe prima observație și apoi să o numere încă o dată în likelihood. Orice schimbare de priori după dezvoltare este înregistrată; testul final nu este folosit pentru alegerea lor.

Dacă neural residual nu este identificabil sau nu aduce beneficiu, release-ul poate selecta V1 cu noul likelihood, cu acest fapt afișat. Nu prezentăm un V1 drept V2 complet validat.

---

<a id="chapter-07"></a>

# 7. Likelihood de observație coerent pe timpi neregulați

## 7.1. Operatorul de observație

Pentru FLEX:

\[
y_j=s_g\sqrt{q(t_j+\Delta t_g)}+\delta_g+e_j.
\]

Scalele, offset-urile și timpii de calibrare sunt comuni acolo unde instrumentul sau sesiunea le face comuni. Nu tratăm o eroare de scară ca zgomot independent pe fiecare frame. Cerințele de proiectare ale camerei nu sunt automat incertitudinea realizată a măsurătorii.

Operatorul se evaluează numai când t_j+Δt_g se află în faza modelată. Un shift negativ care ar cere o stare înainte de intrarea definită nu autorizează extrapolare înapoi: fixăm originea auditată sau excludem acel nuisance setting. Evenimentele și observațiile folosesc aceeași convenție de ceas.

Nu lăsăm liberă, fără ancoră, simultan scara globală, D₀ și originea timpului pe un prefix scurt. Unde lipsesc informații externe, fit-ul este condiționat pe calibrarea publicată și include scenarii de sensibilitate explicit etichetate.

Likelihood-ul se calculează în spațiul diametrului observat. Dacă y=D+e cu E[e]=0, atunci E[y²]=D²+Var(e); a fit-ui y² ca observație nebiased fără adaptarea likelihood-ului ar fi greșit.

## 7.2. Distribuția Student-t multivariată

V1 folosise inovații Student-t condiționate pe o cadență. V3 păstrează corecția V2: o familie de distribuții finite coerentă pe timpi neregulați. [L36](#source-l36)

\[
\mathbf e\sim t_\nu(0,\Sigma),\qquad\nu=4,
\]
\[
\Sigma_{jk}=\sigma_c^2e^{-|t_j-t_k|/\ell}+\sigma_n^2\mathbf1_{j=k}.
\]

Σ este matrice de **scară**. Covarianța este νΣ/(ν−2), dacă ν>2. Construcția exactă este

\[
\omega\sim\operatorname{Gamma}(\nu/2,\nu/2),
\qquad\mathbf e\mid\omega\sim N(0,\Sigma/\omega).
\]

Un singur ω este comun run-ului. Modelul are cozi grele la nivel de run; nu pretinde că fiecare outlier independent este perfect rezolvat. Un track persistent eronat cere reparare/excludere justificată, nu un prior mai larg care ascunde problema.

## 7.3. Log likelihood

Pentru n componente scalare observate, δ=eᵀΣ⁻¹e:

\[
\log L=\log\Gamma\left(\frac{\nu+n}{2}\right)-\log\Gamma\left(\frac\nu2\right)
-\frac n2\log(\nu\pi)-\frac12\log|\Sigma|
-\frac{\nu+n}{2}\log\left(1+\frac\delta\nu\right).
\]

Aceasta este o densitate normalizată. Unitățile și transformările se păstrează consistent când comparăm scoruri. Nu folosim log likelihood absolut ca scor comparabil între variabile cu unități diferite fără convenție comună.

## 7.4. Implementare O(n) cu OU/Kalman

Kernel-ul exponențial are reprezentare Gaussiană de stare la scară unitară:

\[
x_j=\varphi_jx_{j-1}+\eta_j,
\quad\varphi_j=e^{-\Delta t_j/\ell},
\quad\eta_j\sim N(0,\sigma_c^2(1-\varphi_j^2)),
\]
\[
e_j=x_j+\epsilon_j,\qquad\epsilon_j\sim N(0,\sigma_n^2),
\quad x_1\sim N(0,\sigma_c^2).
\]

O trecere Kalman produce inovațiile v_j și varianțele F_j. Atunci log|Σ|=ΣlogF_j, iar δ=Σv_j²/F_j. Acestea intră în formula Student-t de mai sus. Nu simulăm zgomot Gaussian în locul unui Student-t; folosim algebra Gaussiană pentru determinantul și forma pătratică ale aceleiași matrice.

Calculăm 1−exp(−2Δt/ℓ) cu `-expm1(-2Δt/ell)` pentru timpi apropiați. Timestamps identice conflictuale trebuie rezolvate în date; nugget-ul nu este o scuză pentru a număra duplicate drept informație nouă.

Dacă avem o componentă instrumentală Gaussiană cu varianță cunoscută **neîmpărțită la ω**, suma nu mai este exact același Student-t. Păstrăm ω latent și evaluăm likelihood-ul corespunzător; nu folosim formula închisă incorect.

## 7.5. Condiționare exactă

Împărțim observațiile în A și B. Cu n_A=|A| și δ_A=e_AᵀΣ_AA⁻¹e_A:

\[
e_B\mid e_A\sim t_{\nu+n_A}\left(
\Sigma_{BA}\Sigma_{AA}^{-1}e_A,
\frac{\nu+\delta_A}{\nu+n_A}
[\Sigma_{BB}-\Sigma_{BA}\Sigma_{AA}^{-1}\Sigma_{AB}]
\right).
\]

Aceeași formulă se aplică unei observații încă ascunse, dar aflate înainte de cutoff. Ea nu este independentă de punctele deja citite numai fiindcă utilizatorul a apăsat alt buton.

Pentru extinderea unui prefix de la τ₁ la τ₂, greutatea incrementală este:

\[
\Delta\log w=\log L(Y_{\le\tau_2})-\log L(Y_{\le\tau_1})
+\log S(\tau_2)-\log S(\tau_1),
\]

plus corecțiile de propunere și suport necesare. Pentru o observație nouă la același τ, supraviețuirea nu se numără din nou. Mutarea sliderului sau resampling-ul grafic nu schimbă distribuția probabilistică a unei predicții deja emise.

## 7.6. Inventarul incertitudinii

| Componentă | Tratament | Limită |
|---|---|---|
| Calibrare geometrică/temporală | Nuisance comun cu ancoră externă sau sensibilitate | Nu identificăm precizia absolută numai dintr-un trace |
| Eroare de măsurare reziduală | Kernel temporal, nugget, cozi grele | Bias sistematic neobservat poate rămâne |
| Variabilitate între runs | Efecte locale ierarhice | Nu identifică toate cauzele fizice |
| Parametri globali | Posterior sau ansamblu declarat | Corectitudinea clasei de model nu este garantată |
| Integrare numerică | Referință independentă și rafinare | Nu măsoară eroarea fizică |
| Inferență finită | ESS, tails, replici Monte Carlo, referință MCMC | Un ESS bun nu dovedește singur absența bias-ului |
| Etichete/cenzurare | Interval/cause set și sensibilitate | Chimie nevăzută sau politică nemăsurată nu devin identificate |
| Domain shift | Refuz și analiză de suport | Nu există garanție pentru alt combustibil/geometrie/gravitație |

Benzile latente descriu starea fizică din model. Benzile pentru măsurători includ zgomotul și calibrarea. Acoperirea datelor zgomotoase se evaluează față de distribuția **measurement-predictive**, nu față de o bandă latentă mai îngustă.

---

<a id="chapter-08"></a>

# 8. ACME — evenimente dependente de configurație și protocol

## 8.1. Ținta

Pentru o flacără stabilită în configurația normal sau inverse, estimăm timpul și cauza finalului în condițiile și politica experimentală observate. E = încetare spontană adjudicată a fazei; O = terminare de către operator. Pierderea administrativă a observației este tratată separat.

Nu interpretăm FT ca „flacăra ar fi continuat pentru totdeauna”. Nu eliminăm operatorul din date și apoi pretindem că am identificat viața naturală. Pentru acel contrafactual ar trebui istorii de intervenție, ipoteze cauzale, overlap și analize de sensibilitate suplimentare.

## 8.2. Caracteristicile rol-aware

Opt caracteristici nominale per configurație:

1. log(p/p_ref);
2. log(mdot_injected/mdot_ref), cu fluxul injectat definit fizic;
3. fracția de oxigen în rolul explicit al configurației;
4. fracția de combustibil în rolul explicit al configurației;
5. Tad normalizat, numai dacă este calculat din condiții inițiale admise;
6. Zst în aceeași convenție;
7–8. indicatoare pentru combustibil, cu referința declarată.

Normal: fluxul injectat este combustibil. Inverse: fluxul injectat este oxidant. Nu substituim `HC burnrate` derivat post-factum dacă modelul cere setpoint-ul injectat. Scalele mdot_ref și ale temperaturii sunt fixate în manifest din protocol sau training, nu din test. Missingness critică produce restricție de cohortă, nu completare cu valori tipice.

Nu includem TFP până nu este definit, peak burner temperature, compoziție after-test sau durata finală. Indicatorul de aprindere este criteriu de populație, nu o etichetă discretă adăugată pentru a explica retrospectiv un eveniment.

## 8.3. Hazarduri neurale aditive

Pentru k∈{E,O}:

\[
\lambda_{k,i}(t)=t_{ref}^{-1}\exp\left[
a_{k,b(t)}+\beta_k^Tc_i+
\tfrac12\{H_\phi(c_i,\log(1+t/t_{ref}))_k-H_\phi(0,\log(1+t/t_{ref}))_k\}
+u_{g(i),k}\right].
\]

H este o rețea 9→8→2 cu tanh, deci 9·8+8+8·2+2=98 ponderi. Sunt șase bin-uri temporale per cauză, 12 intercepturi de bază, 16 coeficienți liniari și trei parametri ai covarianței frailties: **129 populație/configurație**. Cele două frailties per sesiune au prior Gaussian corelat.

Bin-urile sunt intervale de timp fizic fixate din design/dezvoltare. Nu împărțim t la durata eventuală a run-ului. Modelul de control este static numai pentru intervalele unde această aproximație este admisă. Un control variabil necesită istoricul lui; o schimbare viitoare neanunțată nu este cunoscută predictorului.

Fit-uim două instanțe independente normal/inverse. Sharing-ul suplimentar rămâne un experiment de transfer separat, nu o economie făcută prin amestecarea coloanelor.

## 8.4. Likelihood și cenzurare

S, F_E, F_O și likelihood-ul cause-specific folosesc formularea din capitolul FLEX, cu endpoint-uri ACME. Un interval cu E sau O neclar poate contribui suma densităților numai dacă sursa justifică acel set și intervalul. „Nu știm” nu devine un al treilea mecanism curat.

Un efect de sesiune pentru politica operatorului nu elimină automat politica neobservată, încălzirea arzătorului sau diferențele dintre echipaje. Populația și tipul de generalizare trebuie declarate: sesiune nouă, condiții noi în suport sau nouă observație în sesiune cunoscută.

Pentru o sesiune cunoscută folosim posteriorul ei deja învățat, cu dependența față de Θ. Pentru o sesiune nouă folosim distribuția populației. Nu resetăm aceeași calibrare/frailty la un prior independent pentru fiecare întrebare.

## 8.5. Oprirea programată determinist — completare canonică

Dacă protocolul, cunoscut la cutoff, oprește debitul la t_stop, aceasta nu este o probabilitate continuă de eveniment pe care trebuie să o inventeze rețeaua. Pentru t<t_stop, folosim hazardurile spontane și eventualele intervenții nescheduled admise. Masa rămasă imediat înainte de oprire produce un **atom la t_stop**:

\[
P(T=t_{stop},J=O\mid\text{prefix})
=E\left[\mathbf1_{B\ge t_{stop}}\frac{S(t_{stop}^-)}{S(\tau)}\right]
\]

atât timp cât traiectoria este în suport până acolo. După oprire, probabilitatea de a fi încă în aceeași fază alimentată este zero în acest model al politicii. Nu extrapolăm hazardul netratat și nu numim acea extrapolare „ce ar fi făcut focul fără operator”.

Pentru un record interval-cenzurat care include stopul programat, likelihood-ul adaugă această masă discretă numai dacă O este în setul cauzelor admise, în plus față de integralele continue permise. Nu aproximăm un atom printr-un hazard arbitrar foarte mare.

Dacă toate intervențiile documentate sunt strict programate și nu există un mecanism nescheduled identificabil, nu fit-uim un hazard continuu O numai ca să păstrăm aceeași schemă: îl eliminăm, actualizăm numărul parametrilor și păstrăm atomii cunoscuți. Modelul nominal de129 parametri este pentru cazul în care ambele cauze continue sunt justificate de date.

O terminare **a înregistrării** la o oră programată poate fi cenzurare administrativă; o terminare **a debitului** schimbă sistemul fizic și este O. Cele două sunt diferite chiar dacă timestamp-ul este același.

Pentru hazarduri constante λ_E,λ_O pe un interval Δt înainte de stop, Λ=λ_E+λ_O:

\[
S_{new}=S_{old}e^{-\Lambda\Delta t},\qquad
\Delta F_k=S_{old}\frac{\lambda_k}{\Lambda}(1-e^{-\Lambda\Delta t}).
\]

La Λ≈0 folosim limita stabilă și `expm1`. Aceasta oferă un caz analitic independent pentru teste. În modelul neural, integrarea se face pe intervale de control/bin, cu rafinare și contabilitatea atomilor separată.

## 8.6. Baseline-uri și interpretare

Baseline-ul este hazardul simplu/piecewise și apoi un hazard liniar pe aceleași date. Clasificatorul SE/FT poate fi raportat ca un comparator pentru o întrebare categorială, dar AUC-ul său nu este același metric ca probabilitatea calibrată de eveniment până la un timp.

Curbele UI sunt etichetate „sub politica experimentală observată”. Un rezultat din normal nu este afișat drept predicție inverse. O sesiune fără cauze suficient documentate poate susține evidență descriptivă, nu două curbe precise fabricate.

---

<a id="chapter-09"></a>

# 9. BASS — metrologie și operator neuronal al frontului vizibil

## 9.1. Obiectul observat

Într-o proiecție planară calibrată, reprezentăm frontul luminos leading prin f(s,t), unde s∈[0,W] este coordonata transversală, iar f este poziția longitudinală în metri. Pentru primul release cerem un contur single-valued, un material și un protocol calificat. Nu presupunem că f este o izotermă ori frontul chimic de piroliză.

O transformare de cameră nu elimină automat parallax-ul volumetric al flăcării. Dacă abaterea de la plan nu poate fi mărginită, raportăm geometrie proiectată sau pixeli, nu o măsurare fizică cu precizie fictivă. O desprindere a flăcării sau o ocluzie este status de măsurare/suport, nu un motiv să desenăm un contur complet prin interpolare nejustificată.

## 9.2. Pipeline-ul vizual

Primul comparator este o metodă clasică de segmentare/edge + geometrie. SAM2.1 este un ajutor pretrained pentru măști temporale, nu un instrument calibrat prin simpla sa utilizare. [L16–L17, T18–T19, T28]

Pașii sunt: decode PTS → corecție distorsiune → crop/ROI admis → segmentare → boundary definition → transformare geometrică → contur/uncertitudine → quality flags. Referințele se distribuie pe run-uri, condiții și situații dificile. Un al doilea model care este de acord cu primul nu este singur suficient ca adevăr metrologic.

Există două căi separate:

**Retrospectivă:** curatorul poate privi întregul clip pentru corectarea identității, calibrării și anotărilor de referință.

**Cauzală:** inferența la τ folosește numai cadre/prompturi permise până la τ și numai observațiile admise în acel benchmark. Starea tracker-ului se poate reconstrui din acel subset. Nu moștenește corecții retrospective din sufix.

Fine-tuning-ul începe numai dacă eroarea pretrained pe dezvoltare justifică costul. Prima încercare îngheață backbone-ul și adaptează numai componente suportate de implementarea oficială. Activările video, rezoluția și lungimea clipului controlează memoria; 24 GB se verifică prin pilot.

## 9.3. Coordonate și normalizare fixate

Definim ξ=s/W și η=f/L. Derivatele fizice sunt f_s=(L/W)η_ξ și f_ss=(L/W²)η_ξξ. Nu introducem într-o rețea când derivate în metri, când derivate adimensionale sub același nume.

Folosim K=32 noduri ξ_j=j/(K−1), j=0,…,K−1. Cele opt canale inițiale sunt:

1. η;
2. η_ξ;
3. η_ξξ;
4. ξ;
5. masca statică a domeniului geometric admis;
6. x_min(s)/L, marginea cunoscută de intrare a probei;
7. x_max(s)/L, marginea cunoscută de ieșire;
8. distanța la marginea laterală relevantă, normalizată cu W.

Canalele 5–8 sunt funcții determinate din geometria probei/calibrare, nu măști de vizibilitate din viitor. Pentru un dreptunghi, unele sunt constante; această redundanță nu adaugă informație și poate fi eliminată printr-o revizie declarată a tensorului și a numărului de parametri.

Cele opt condiții nominale pentru FiLM sunt: oxigen normalizat, log presiune, magnitudinea vitezei de curgere normalizate, semnul direcției curgerii, log lățime, log lungime, log grosime și timpul de la ultima schimbare de control **cunoscută**. Scalele geometrice și de viteză sunt fixate din protocol/train. Direcțiile și dimensiunile constante se pot elimina. Nu inventăm presiune, grosime sau airflow pentru a completa vectorul.

O schimbare viitoare a ventilatorului intră în forecast numai dacă este un program cunoscut la cutoff. Altfel, forecast-ul este pentru scenariul declarat de control constant ori se oprește înaintea unui interval neidentificat; nu primește logul viitor real în secret.

## 9.4. Operatorul integral neperiodic

Convenția de normalizare este explicită [IGNIS]. Ponderile trapezoidale pe [0,1] sunt q_0=q_{K−1}=1/[2(K−1)] și q_j=1/(K−1) pentru interior. Definim

\[
\varphi_0(\xi)=1,\qquad\varphi_m(\xi)=\sqrt2\cos(m\pi\xi),\quad m=1,2,3.
\]

Coeficienții sunt

\[
\widehat h_m=\sum_jq_j\varphi_m(\xi_j)h(\xi_j),
\qquad
\mathcal K_\ell h(\xi)=\sum_{m=0}^3\varphi_m(\xi)A_{\ell m}\widehat h_m.
\]

Această convenție face verificabil modul constant și evită factori ascunși de doi. Implementările Rust/JAX trebuie să folosească aceleași ponderi și aceeași convenție, nu două funcții DCT cu normalizări implicite diferite.

După lift 8→16, trei blocuri:

\[
h_{\ell+1}(\xi)=\tanh\left((1+\gamma_\ell(c))\odot
[W_\ell h_\ell(\xi)+\mathcal K_\ell h_\ell(\xi)+b_\ell]+\beta_\ell(c)\right).
\]

FiLM 8→16→96 produce γ și β pentru trei blocuri de lățime 16. Un head liniar 16→1 produce N_θ. Numărul ponderilor este 144 lift +3·(4·16²+16²+16)=3888 blocuri +17 head +1776 FiLM = **5825**.

Cosine modes evită wrap-around-ul periodic. Ele nu reprezintă, prin ele însele, dovada unei condiții fizice Neumann la marginile probei. Convenția diferențelor și tratamentul marginilor se declară separat.

## 9.5. Dinamica cinematică

\[
\partial_tf=
[v_b(s,c,b)+v_r\tanh N_\theta(f,c,G)]\sqrt{1+(\partial_sf)^2}
+\nu_f\frac{\partial_{ss}f}{1+(\partial_sf)^2},\quad\nu_f\ge0.
\]

v_b este baseline-ul de viteză, v_r o scală a rezidualului, iar ν_f regularizare cinematică de curbură, în m²/s. Nu o interpretăm drept difuzivitate termică măsurată. Semnul pozitiv al termenului de curbură este difuziv. Conturul vizibil poate avansa sau se poate retrage; nu impunem monotonicitatea unui front de consum chimic pe un observabil luminos.

Starea locală nominală are opt coeficienți ai conturului inițial și patru coeficienți de variație a vitezei. O implementare simplă folosește patru baze spațiale joase declarate pentru variația locală. Priorul, viteza de referință și parametrii baseline sunt numărați separat de cei 5825 ai rețelei. Nu reutilizăm primul contur ca prior îngust și ca observație independentă încă o dată.

### Parametrizare executabilă a baseline-ului și stării locale [IGNIS]

Pentru a nu lăsa simbolul v_b nedefinit în implementare, convenția nominală V3 este

\[
v_b(s,c,b)=v_{ref}\left[\beta_v^\top(1,c)+\sum_{m=0}^{3}b_m\varphi_m(s/W)\right],
\quad v_{ref}=10^{-3}\,\mathrm{m/s},\quad v_r=v_{ref}.
\]

Cele nouă β_v sunt parametri globali MAP. v_ref este o scară numerică de pornire, nu o viteză măsurată. Patru b_m sunt efecte locale cu priori Gaussieni cu scale inițiale fixe 0,25/(1+m), supuse unei sensibilități de dezvoltare. Constantele de prior nu se numără drept parametri fit.

Frontul inițial este parametrizat în coordonate neconstrânse:

\[
f_0(s)=L\,\operatorname{sigmoid}\left(\sum_{m=0}^{7}a_m\varphi_m(s/W)\right).
\]

Această convenție asigură f₀∈(0,L), nu o geometrie chimică adevărată. a₀~N(0,1), iar a_m~N(0,[0,5/m²]²) pentru m≥1 sunt priori de proiectare, nu deduși din prima măsurătoare. Likelihood-ul primului contur îi informează o singură dată. O cohortă al cărei domeniu geometric diferă cere o parametrizare adaptată declarată.

Regularizarea este ν_f=ν_ref exp ζ, ν_ref=10⁻⁶ m²/s, ζ~N(0,1) inițial. Ea nu reprezintă o proprietate termică. Cu trei parametri ai kernel-ului observațional și geometria spațială a erorii fixată din metrologie/dezvoltare, inventarul nominal per membru este **5825 ponderi NN +9 β_v +1 ζ +3 parametri observaționali =5838 parametri globali fit**, plus 12 parametri locali per run în fitting-ul MAP. Nuisances de calibrare justificate se numără separat. Această completare precizează numărul care în V2 era lăsat în afara headline-ului de 5825 ponderi.


## 9.6. Numerică: limite care nu dispar prin bounded NN

Un tanh la ieșire limitează viteza reziduală, dar nu demonstrează stabilitatea PDE-ului discret. Pantele, curburile, feedback-ul rețelei și marginile contează.

Pentru implementare alegem un integrator explicit de ordinul doi pe domeniul smooth-front admis, cu flux de tip upwind/Lax–Friedrichs pentru partea de transport și un stencil difuziv consistent pentru curbură. Nu folosim fără verificare diferențe centrale nedisipative pentru o componentă de transport și apoi declarăm stabilitate numai din condiția CFL.

La fiecare etapă evaluăm viteza pe starea curentă. Un pas conservator este limitat de scalele Δs/v_max și Δs²/(2ν_f), plus un prag pentru feedback-ul local al operatorului. Coeficienții de siguranță sunt setări numerice testate; aceste limite nu reprezintă o teoremă de stabilitate pentru orice rețea învățată.

În primul domeniu calificat se folosesc margini documentate: ROI interior sau extrapolare liniară a valorilor ghost, cu sensibilitate la o alternativă justificată. Nu tratăm această convenție numerică drept proprietate universală a combustiei la margine. Testăm 32 versus 64 noduri și Δt versus Δt/2 pe dezvoltare. Dacă rezultatele se schimbă material, modelul nu este calificat la rezoluția ieftină.

## 9.7. Observație și training

Contururile sunt proiectate pe opt coeficienți spațiali fixați pentru likelihood; păstrăm toate nodurile și reziduurile pentru scoruri geometrice. O eroare mare în modurile mai înalte nu poate fi ascunsă prin evaluarea exclusivă a primilor opt coeficienți.

Folosim un likelihood temporal/spațial declarat, cu calibrare comună per cameră/sesiune. O primă implementare poate folosi o covarianță spațială fixată din erorile metrologice de dezvoltare și kernel temporal Student-t. Nu estimăm arbitrar o matrice 32×32 liberă din puține run-uri.

Cele cinci modele MAP sunt antrenate pe bootstrap de **grupuri**, nu de pixeli. Numărul total al ponderilor NN este 29125, dar cele cinci modele nu sunt cinci dataseturi independente. Răspândirea ansamblului nu reprezintă toată incertitudinea epistemică; se adaugă măsurare, calibrare și inferență locală.

Scorurile obligatorii: distanța geometrică în metri/mm, erori pe contur/run, bias de scară, drift al tracker-ului, rata ocluziilor/eșecurilor și skill față de persistență/viteză constantă. Comparăm pe clipuri/runs/specimene nefolosite la fitting. Nu publicăm „accuracy 99%” derivată din majoritatea pixelilor de fundal.

---

<a id="chapter-10"></a>

# 10. Inferența din prefix și acceleratoarele neuronale

## 10.1. Calea obligatorie nu depinde de o rețea de propuneri

Mai întâi implementăm likelihood-ul și condiționarea corectă. Folosim sampling din prior/populație cu reponderare, o propunere parametrică justificată sau rafinare locală MCMC. Abia după profilare antrenăm un encoder/flow pentru familia în care inferența lentă afectează produsul.

Economia nu constă în omiterea corecției probabilistice. Dacă o aproximație nu are o densitate evaluabilă sau schimbă ținta, ea nu poate fi prezentată ca același posterior doar pentru că este rapidă.

### Calea inițială ACME: cuadratură Gaussiană în două dimensiuni

Pentru o sesiune nouă, singurele efecte locale ACME nominale sunt u_E,u_O, Gaussiene corelate. Înainte de a antrena două flows, folosim **Gauss–Hermite bidimensional**, cu Cholesky L_g al covarianței pentru fiecare draw global g. Dacă x_i,w_i sunt nodurile/ponderile pentru exp(−x²), atunci u_ij=√2 L_g(x_i,x_j) și greutatea de prior este w_iw_j/π.

Greutățile joint de prefix sunt

\[
W_{gij}\propto w_g^{train}\frac{w_iw_j}{\pi}
L(Y_{\le\tau}\mid\Theta_g,u_{ij})S_g(\tau\mid u_{ij})\mathbf1_{support}.
\]

**Normalizăm peste g,i,j împreună**, nu separat în fiecare draw global lăsat apoi cu pondere egală. Prefixul informează și distribuția globală. Într-un event-only expert, L_obs poate fi unu; informația nouă este supraviețuirea observată până la τ și controlurile admise.

Comparăm 12,24 și48 noduri pe fiecare dimensiune pe cazurile de dezvoltare. Pentru frailty constant pe timp, hazardul cumulativ per cauză se separă: H_k(t|u_k)=exp(u_k)H_{k,base}(t), astfel încât calculăm o dată curbele de bază pentru un draw global și reutilizăm pe noduri. Aceasta poate elimina complet nevoia unui accelerator neuronal ACME pentru primul release. Este o optimizare a aceleiași ținte, nu alt model fizic.

Pentru o sesiune cunoscută cu posterior local ne-Gaussian, pentru nuisance-uri suplimentare ori pentru o cuadratură insuficientă, folosim posteriorul de training/particles și referința corectă. Nu impunem o aproximație Gaussiană falsă doar ca să păstrăm două dimensiuni. Această alegere este noua cale inițială de serving V3; specificațiile flow ACME rămân în raport ca extensie, nu ca job obligatoriu.


## 10.2. Ținta condiționată

În coordonate locale neconstrânse b și cu Θ un draw global sau index de ansamblu:

\[
\pi_\tau(\Theta,b,\kappa)\propto
p_{train}(\Theta,\kappa_{known})p_0(b,\kappa_{new}\mid\Theta)
L(Y_{\le\tau}\mid\Theta,b,\kappa)S(\tau\mid\Theta,b)
\mathbf1_{support}.
\]

Termenul S există numai pentru expertul și observația de activitate corespunzătoare. BASS nu primește artificial hazardul FLEX. Pentru o calibrare cunoscută păstrăm dependența cu posteriorul de training; pentru una nouă folosim priorul justificat.

Un prior Gaussian pe log D₀ este deja o densitate în coordonata neconstrânsă. Dacă samplerul lucrează în log D₀, nu adăugăm încă un Jacobian ca și când priorul ar fi definit direct pe D₀. Convenția este fixată în model manifest.

## 10.3. Encoderul opțional

Schema nominală are tokens de 24 componente: timp și gap normalizate, valoare, scala admisă a măsurării, mask/quantity metadata și control/context, într-o ordine înghețată. Toate provin din prefix. Schema concretă trebuie să enumere fiecare componentă; nu lăsăm un agent să introducă textul finalului într-un embedding nespecificat.

Două blocuri transformer, width64, patru heads, FFN128, layer normalization, pooling mascat și 16 metadate suplimentare formează contextul. Parametri:

| Componentă | Număr |
|---|---:|
| Proiecție 24→64 | 1600 |
| Două blocuri | 66944 |
| Normalizare finală | 128 |
| Proiecție context 80→64 | 5184 |
| **Total encoder** | **73856** |

Attention bidirecțională între observații deja admise este permisă. Atenția asupra viitorului ascuns nu este permisă. Padding-ul nu trebuie să influențeze pooling-ul, normalizarea ori densitatea.

### Schema nominală exactă a tokenului și contextului

Tokenul de24 componente are: (0) log1p(t/t_ref), (1) log1p(gap/t_ref), (2) valoare normalizată a cantității, (3) log scară de măsurare normalizată dacă este cunoscută, altfel zero împreună cu masca; (4) indicator că scara este cunoscută; (5) indicator valid; (6) confirmare de fază disponibilă atunci; (7) indicator al unei schimbări de control cunoscute; (8–15) one-hot pentru opt coduri de canal/mode în schema familiei; (16–23) cele opt condiții role-aware cunoscute la observație, cu padding declarat unde există mai puține.

FLEX folosește numai canalele admise de propriul likelihood; un canal vizual suplimentar nu devine predictor fizic implicit. BASS poate reprezenta cele opt moduri ca tokens distincte cu același timestamp. ACME event-only poate avea un token de stare/activitate; nu inventează măsurători continue. Timestamps coincidente ale canalelor nu înseamnă duplicarea aceleiași observații în likelihood.

Cele16 metadate adăugate după pooling sunt condițiile curente8 și: cutoff, log1p(număr de observații admise), primul/ultimul timp admis, fracția lipsurilor față de programul de observație declarat, flag de calibrare spațială, flag de calibrare temporală și flag de fază confirmată. Toate transformările și valorile lipsă sunt în manifest.

Cele16 caracteristici ale draw-ului global, folosite de conditioner, sunt inițial o proiecție PCA de rang≤16 asupra parametrilor globali în coordonate neconstrânse, standardizați din draws de training. Centrul, scara, loadings și floor-ul numeric sunt artefacte fit exclusiv pe training și nu intră în numărul ponderilor NN. Dacă dimensiunea este mai mică, padding-ul este fix. PCA este numai un rezumat pentru propunere, nu o reducere a Θ în likelihood. Pierderea de informație afectează eficiența propunerii; corecția folosește Θ complet. Dacă această proiecție este slabă din cauza simetriilor ponderilor, se înlocuiește prin features funcționale declarate și se reevaluează propunerea, fără a schimba ținta probabilistică.


## 10.4. Flow spline

Opt coupling layers alternează dimensiunile transformate. Folosim rational-quadratic splines cu opt bin-uri și tails identitate în afara [−5,5] în coordonate locale standardizate. Per componentă transformată există 8 lățimi, 8 înălțimi, 7 derivate interioare =23 outputuri. Lățimile/înălțimile sunt pozitive și normalizate, cu minime compatibile cu numărul de bin-uri. Derivatele la cozi sunt fixate la unu. [L22](#source-l22)

Într-un bin, w lățime, h înălțime, δ=h/w, ξ=(x−x₀)/w:

\[
y=y_0+h\frac{\delta\xi^2+d_0\xi(1-\xi)}
{\delta+(d_1+d_0-2\delta)\xi(1-\xi)},
\]
\[
\frac{dy}{dx}=\frac{\delta^2[d_1\xi^2+2\delta\xi(1-\xi)+d_0(1-\xi)^2]}
{[\delta+(d_1+d_0-2\delta)\xi(1-\xi)]^2}>0.
\]

Densitatea se obține prin change of variables. Inversa, derivativele și log-Jacobianul se verifică independent. **Propunerea poate aproxima posteriorul; densitatea propriei propuneri trebuie calculată corect.**

Conditioner-ul are două straturi ascunse 64/64 și primește componentele nemodificate, context64 și un rezumat determinist de 16 componente al draw-ului global. Rezumatul global poate pierde informație pentru eficiența propunerii, dar likelihood-ul final folosește Θ complet, nu rezumatul.

| Familie | Dimensiune locală nominală | Encoder + flow |
|---|---:|---:|
| FLEX | 3 | 167316 |
| ACME normal | 2 | 161080 |
| ACME inverse | 2 | 161080 |
| BASS | 12 | 223440 |
| **Toate patru** | Nu un singur spațiu latent | **712916** |

Nuisances locale noi pot mări dimensiunea; acestea cer o configurație nouă, nu reutilizarea headline-ului vechi. Cele patru instanțe nu sunt obligatoriu antrenate toate în campania de 100 USD.

## 10.5. Propunere defensivă și reponderare

\[
q_\epsilon(b\mid\Theta,Y)=(1-\epsilon)q_\psi(b\mid\Theta,Y)+\epsilon p_0(b\mid\Theta),
\qquad\epsilon=0.1\text{ inițial}.
\]

Draw Θ din p_train și b din amestec. Atunci:

\[
\log w=\log p_0(b\mid\Theta)+\log L(Y\mid\Theta,b)+\log S(\tau\mid\Theta,b)
-\log q_\epsilon(b\mid\Theta,Y)+\log\mathbf1_{support}.
\]

Densitatea globală se anulează numai dacă propunerea Θ este aceeași distribuție. Dacă selectăm altfel indicii globali, includem raportul corespunzător. Densitatea amestecului include ambele componente indiferent care a generat particula; calculăm cu log-sum-exp.

După normalizare, ESS=1/Σw̄². Verificăm și greutatea maximă, tails, variația între seeds, outputurile relevante și diferența față de condiționarea de referință. ESS bun este necesar, nu suficient. Un criteriu de pornire poate declanșa rafinare sub 20% din particule, dar acel procent este setare inginerească, nu teoremă de precizie.

Rafinarea poate crește particulele sau folosi un kernel invariant față de aceeași țintă. Nu schimbă likelihood-ul pentru a produce o bandă mai îngustă. Corecția de importanță este asimptotică, cu condiții de suport; nu transformă un model fizic greșit într-unul adevărat.

## 10.6. Training fără a inventa date NASA

După fitting-ul unui expert numai pe dezvoltare, generăm sarcini statistice: Θ, efecte locale, programe de observație, controale admise, zgomot și evenimente conform modelului. Cutoff-ul se alege printr-o regulă independentă de durata finală; condiția de activitate/suport se păstrează consistent în distribuția sarcinilor.

Rețeaua se antrenează cu negative log density a latentului cunoscut. Dacă sampling-ul taskurilor este schimbat, distribuția sa trebuie documentată; runtime-ul rămâne corectat față de ținta explicită. [L23](#source-l23)

Bugetul mare anterior de 3,2 milioane taskuri =4 instanțe ×4 fits ×200000 **nu mai este obligatoriu**. Începem cu o instanță, aproximativ 100000 taskuri și un plafon de pași, apoi verificăm ESS/latency/eroare. O extensie nu adaugă experimente NASA independente și nu justifică afirmații de generalizare fizică.

## 10.7. Cache și actualizări

Cheia conține prefixul exact, model/data/calibration versions, horizon, support policy, solver tolerances, RNG și configurația inferenței. Schimbarea unui singur input relevant invalidează cache-ul.

La o observație nouă, includem numai likelihood-ul condiționat incremental și, dacă timpul avansează, raportul de supraviețuire. Mutarea cutoff-ului înapoi restaurează o stare anterioară corectă sau recalculează; nu folosește posteriorul ulterior cu graficele ascunse.

---

<a id="chapter-11"></a>

# 11. Decision Intelligence, active observation și Jev

## 11.1. Trei sensuri diferite ale cuvântului „decizie”

**Rutare semantică:** ce tool/familie răspunde întrebării? Aici poate ajuta Jev.

**Selecție de informație:** ce măsurătoare merită inspectată, dată fiind o pierdere predictivă și un cost? Aici folosim distribuțiile experților și calcul numeric.

**Decizie operațională de siguranță:** ce material să zboare, cum să fie operat un habitat sau ce intervenție asupra unui incendiu real să se facă? Aceasta nu este autorizată sau validată de datele și modelele Ignis.

Confundarea acestor niveluri ar produce un demo aparent inteligent, dar științific neclar.

## 11.2. Acțiunea admisă în benchmark

O acțiune selectează o măsurătoare încă neinspectată la un timestamp **≤τ**, ori o acțiune de anotare/calibrare cu un model de informație declarat. Viitorul t>τ rămâne sigilat. Nu comandăm experimente fizice și nu numim replay-ul arhivei o demonstrație de control în timp real.

Toate politicile pornesc cu aceleași observații și același buget. Un frame ascuns nu a influențat deja tracker-ul prefixului. Acțiunile deja citite, imposibil de calibrat ori din afara domeniului sunt eliminate deterministic înainte de scoring.

### Ce experți pot participa efectiv

Primul benchmark numeric de achiziție este **FLEX**. BASS intră după ce are un likelihood de contur/calibrare calificat. **ACME event-only nu oferă automat o observație nouă utilă:** dacă știm deja că aceeași fază a rămas activă până la τ, întrebarea dacă era activă la un timp anterior poate fi redundantă. Nu inventăm un senzor sau un rezultat de utilitate pentru a avea același buton în toate cardurile.

Un expert expune `rank_observations` numai dacă definește distribuția măsurătorii candidate și felul în care ea intră în likelihood. Pentru ACME, inspectarea unui log pentru a rezolva un endpoint poate fi o acțiune de curator/evidence, dar nu este prezentată drept achiziție online pentru același forecast fără un model explicit al informației. Extinderea la condiții incerte observabile cere prior și observation model pentru acele condiții.

## 11.3. Bayes risk și utilitate

Cu ținta Z și decizia de estimare d:

\[
\mathcal R(\mathcal D)=\min_d E[\ell(d,Z)\mid\mathcal D].
\]

Pentru acțiunea a și măsurătoarea potențială Y_a:

\[
U(a)=\mathcal R(\mathcal D)-E_{Y_a\mid\mathcal D}
[\mathcal R(\mathcal D\cup\{Y_a\})]-\lambda C(a).
\]

La squared error, reducerea riscului este reducerea așteptată a varianței țintei, cu ponderile/unitățile declarate. La log score, apare o relație cu informația condiționată. Nu folosim automat entropia parametrilor dacă interesul cercetătorului este eroarea asupra unui rezultat concret. [L19](#source-l19)

Pentru prima implementare alegem un target fix înainte de scoring: de exemplu probabilitatea model-condițională a sfârșitului fazei într-un orizont comun, înainte de ieșirea din suport. Pentru analiza geometrică, ținta poate fi un vector de coeficienți ai frontului ori diametrul într-un interval în care predicția este bine definită. **O stare care nu există după un eveniment nu este completată arbitrar cu zero.** Scorurile de eveniment, de geometrie condiționată și masa nerezolvată sunt păstrate distinct.

O implementare robustă a utilității poate folosi vectorul de probabilități model-condiționale (A,F_H,F_R,U) la un h fix și o pierdere quadratică ponderată asupra lui. Aceasta măsoară incertitudinea asupra unei prognoze definite, nu pretinde observarea directă a parametrilor. Beneficiul empiric se judecă separat, pe evenimente și continuări reale cu cenzurarea tratată corect. Nu recompensăm o politică pentru simpla ascundere a cazurilor din suport.

## 11.4. Calcul bounded

Setarea inițială din designul mare era 32 acțiuni, 16 observații ipotetice per acțiune și 2048 particule. Sub bugetul curent începem cu 8–16 acțiuni bine definite și 8–16 rezultate ipotetice, apoi rafinăm numai dacă ranking-ul este instabil. Aceste numere nu sunt garanții de precizie.

Observațiile ipotetice se generează din distribuția comună, condiționată pe erorile/calibrarea deja observate. Punctele adiacente nu sunt independente. Refolosim traiectoriile și common random numbers unde este corect pentru a reduce zgomotul comparației. Dacă două acțiuni au utilități apropiate față de eroarea Monte Carlo, raportăm o egalitate practică ori alegem cea mai ieftină, nu pretindem optimul exact.

Costul C poate include timp de anotare măsurat sau timp de calcul. λ se fixează după unitatea aleasă înainte de test, nu se ajustează ca să câștige politica în demo.

## 11.5. Evaluare

Politici: random, uniform/latest, incertitudine simplă și utilitatea modelată. Bugete: una și, dacă resursele permit, trei observații. Acțiunile se execută pe valorile reale ale arhivei după alegere; viitorul se dezvăluie numai după ce forecast-ul actualizat a fost înghețat.

Raportăm diferența de proper score/eroare, costul, incertitudinea efectului între grupuri și eșecurile. O observație surprinzătoare poate mări incertitudinea sau înrăutăți un forecast individual. Nu animați fiecare update ca pe o îmbunătățire garantată.

În primul release folosim planificare pe un pas. POMDP/multi-step rămâne extensie: starea ar fi belief-ul, acțiunile observațiile, tranziția actualizarea Bayesiană, iar recompensa reducerea loss-ului minus cost. Această formulare nu cere automat training RL. Un Decision Transformer are nevoie de date și obiectiv de politică pe care nu le-am stabilit aici.

## 11.6. Jev generalist: integrare permisă, dar neforțată

Echipa a propus Jev pentru decision models. Îl delimităm ca adaptor API de rutare semantică: `search_evidence`, `compare_experiments`, `forecast_flex`, `forecast_acme`, `forecast_bass`, `rank_observations`, `review_required`. [J01–J03]

Gate-urile deterministe controlează unitățile, fizica, permisii, existența calibrării, versiunea modelului și costul. Un scor de decizie Jev nu este probabilitate de stingere și nu este interval de incertitudine fizică.

Versiunile 1.13/1.14 și prețul de 0,042 USD/milion input, menționate anterior, sunt **istorice și de reconfirmat**, nu defaulturi garantate în această revizie. Nu selectăm un alias care își poate schimba comportamentul între validare și demo. Nu presupunem fine-tuning client-specific sau API gratuit datorită abonamentelor existente.

Testul propus are 300 de cereri revizuite: 200 dezvoltare, 100 test, grupate după familii de formulare ca să evităm aproape-duplicate. Comparăm corectitudinea rutei, false acceptances, abținerea, latența și costul cu regulile/routerul existent. Dacă UI-ul a selectat deja expertul și operația, apelul Jev este redundant și nu se face.

Jev este eliminabil fără a afecta matematica sau dashboardul. Orice cost suplimentar neacoperit intră în rezerva de 27,88 USD. Dacă nu aduce un beneficiu măsurabil, nu îl activăm.

---

<a id="chapter-12"></a>

# 12. Campania de training sub plafonul de 100 USD

## 12.1. Ce este obligatoriu și ce este condiționat

**Obligatoriu pentru o afirmație predictivă:** date calificate, baseline, candidat pe aceeași problemă, verificarea likelihood-ului/solverului, diagnostic al inferenței, evaluare pe grupuri independente și raportul eșecurilor.

**Condiționat de beneficiu:** fine-tuning SAM2, rețeaua de propuneri pentru fiecare familie, dimensiuni mai mari de model, alte folds, GPU serving, Jev și kernel-uri personalizate.

**Nu finanțăm în campania de bază:** training de LLM, universal simulator, generare CFD masivă nevalidată, RL generalist, sweep de zeci de arhitecturi sau GPU permanent pe durata dezvoltării.

## 12.2. Protocolul experimental înghețat

Registrul prevede întrebarea, populația, targetul, condițiile/controalele permise, grupurile, splitul, baseline-urile, metricile, limitele de suport, loss-ul decizional și regulile de alegere a demonstrației.

Un split grupat train/validation/test înlocuiește campania exhaustivă cu trei folds pentru toate modelele. Acesta este un compromis explicit privind robustețea între splituri. Nu îl prezentăm ca echivalent metodologic cu întregul plan de 750 USD.

Selecția NN versus baseline și early stopping se fac pe validation. Fit-ul final poate folosi train+validation după ce alegerea este înghețată; testul rămâne neatins. În acest caz nu pretindem calibrare conformală independentă dintr-un set deja absorbit în fit. Intervalele rămân model-condiționale, cu acoperirea raportată ulterior pe test. Dacă există suficiente grupuri pentru un set separat de calibrare, se poate păstra, dar nu inventăm independență pentru a satisface o etichetă statistică.

## 12.3. Etapele FLEX

1. Verificare de identitate, unități și fază; flow table complet.
2. Baseline D² condiții/populație și baseline de rată din prefix, cu incertitudine de măsurare.
3. V1 fără rezidual, cu likelihood Student-t coerent.
4. V2 cu rezidual bounded și aceleași observații/evenimente.
5. Diagnostic, verificare de priori și o sensibilitate prestabilită.
6. Alegerea candidatului pe validation; fit de release; predicții finale sigilate.

Setarea inițială NUTS moștenită este patru chains, 1000 warmup și 1500 samples/chain, `target_accept_prob=0.9`, tree depth10. Acestea sunt puncte de pornire, nu dovezi de convergență. Pilotul poate impune reparametrizare sau altă alocare, fără să declare valid un lanț insuficient doar pentru a respecta orele.

Folosim full-batch likelihood float64, efecte non-centered și operații JAX compilate. Nu minibatch-uim cadre aleatorii în NUTS obișnuit. Pentru eficiență, optimizăm integralele și kernel-ul înainte de a concluziona că avem nevoie de alt GPU. [L37, T01–T07]

## 12.4. Etapele ACME

Adjudicarea cauzei și a timpului precede modelul. Instanțele normal/inverse pornesc cu baseline piecewise/linear; modelul neural este comparat pe aceeași populație. Verificăm rolul frailties, separarea politicilor, conflictele sursei și hazardul la marginea orizontului.

Nu numărăm controlurile multiple dintr-o sesiune ca toate independente pentru intervalele de performanță. Un input cunoscut numai la final rămâne interzis chiar dacă produce un AUC excelent.

## 12.5. Etapele BASS și vision

Prima rundă folosește SAM pretrained și un comparator clasic. Se măsoară eroarea de calibrare/contur înainte de forecasting. Adaptarea se justifică prin reducerea unei erori metrologice pe dezvoltare, nu prin faptul că un GPU este disponibil.

Antrenăm un membru pilot de operator, apoi extindem la cele cinci bootstrap fits dacă problema este evaluabilă și budget-ul o permite. Obiectivul rămâne MAP sub likelihood-ul și priorii declarați, nu un amestec arbitrar de loss-uri de imagine.

Punct de pornire AdamW: learning rate 10⁻³, clip norm1, cel mult 6000 pași/member, batch mic de ferestre din run-uri, monitorizare pe întregul run de validation. Learning rate și pașii sunt setări propuse, nu valori optime demonstrate. Sampling-ul ferestrelor este echilibrat între runs astfel încât un clip foarte lung să nu definească singur prioritatea empirică. Valoarea exactă a loss-ului raportat trebuie să specifice dacă este per run sau per observație.

Rezoluția32 și pasul temporal au o singură verificare de rafinare înainte de final. Nu facem automat 25 fits și trei folds ca în vechiul scenariu. Limitele acestei campanii se declară.

## 12.6. Training de accelerator numai după profiling

Pentru ACME încercăm întâi cuadratura în două dimensiuni descrisă în capitolul10; nu consumăm bugetul automat pe două flows. Pentru primul flow admis, pornim cu aproximativ 100000 taskuri statistice și un plafon de 10000–25000 pași, FP32, batch determinat prin pilot. Alegerea numărului de taskuri nu este rezultatul unui test pe NASA. Nu mutăm generarea pe GPU dacă integrarea float64 mică este mai eficientă pe CPU.

Criteriul de continuare este costul per forecast calificat: latență incluzând corecția, ESS, Monte Carlo error, rata rafinării și acordul cu referința. Un flow care reduce numai timpul de forward, dar provoacă multe refaceri MCMC, poate fi o pierdere.

## 12.7. Preflight și experiment registry

Înainte de plata ferestrei mari, avem codul local testat, datele staged și un job manifest reproductibil. Preflight-ul de 2–4 ore este inclus în buget și verifică hardware-ul real, compilarea, memoria, I/O și un mini-workload pentru fiecare framework.

Nu presupunem că patru lanțuri pe o placă sunt patru dispozitive independente. NumPyro/JAX trebuie configurate și măsurate pentru concurența reală. Pilotul scurt de câteva sute de tranziții dimensionează costul; nu califică posteriorul de release.

`experiment_registry` conține și ipotezele eșuate, nu doar cel mai bun run. Seed-ul, codul, datele, device-ul, timpul și costul se salvează împreună cu checkpoint-ul.

## 12.8. Ordinea joburilor

Datele și testele se fac local în paralel. Pe pod: pilot → fitting CPU FLEX/ACME concomitent cu metrologie GPU în limite măsurate → BASS → prima accelerare justificată → benchmark decizional → reproducere și freeze. Qwen serving nu prealocă VRAM în timpul trainingului. Indexarea de documente este un job separat.

Nu există patru persoane care modifică același checkpoint. Fiecare job are un director și un lease. Un singur scheduler limitează GPU-heavy tasks, iar thread pools CPU/BLAS/Rayon/JAX sunt coordonate.

---

<a id="chapter-13"></a>

# 13. Verificare: patru întrebări diferite

1. **Codul calculează ecuația declarată?**
2. **Inferența aproximează adecvat distribuția declarată?**
3. **Distribuția prezice experimente NASA independente?**
4. **Utilizatorul poate interpreta rezultatul fără a fi indus în eroare?**

Niciun răspuns pozitiv nu le implică automat pe celelalte. Un test sintetic PASS nu este validare NASA; o interfață premium nu este benchmark; un sampler stabil nu demonstrează mecanismul fizic.

## 13.1. Suita matematică

| Test | Verificare |
|---|---|
| D² constant-rate | Recuperarea formulei în unități SI și a semnului corect |
| Integrală/inversă | Acord cu o cuadratură/ODE independentă |
| Gradient implicit | Acord cu finite differences în interiorul domeniului |
| Cause-specific interval | Nu confundăm o singură cauză cu diferența de supraviețuire totală |
| Suport | A+U+ΣF=1, inclusiv la limita B |
| Stop determinist ACME | Atomul de probabilitate este contabilizat și supraviețuirea post-stop este corectă |
| Student-t | Log densitate comună = marginală + condițională |
| OU/Kalman | log determinant și forma pătratică egale cu algebra densă |
| Cosine BASS | Modul constant și ortogonalitatea discretă au normalizarea declarată |
| Spline | Inversă, Jacobian pozitiv și densitate corectă |
| Defensive mixture | Densitatea ambelor componente apare în numitor |
| Paritate | JAX/Rust aceleași inputuri, toleranțe și rezultate |
| Inventar | Count de parametri egal cu tensor shapes |

Pachetul acestei consolidări include verificări sintetice executabile pentru un subset al acestor identități și aritmetica bugetului. Paritatea runtime-ului complet, performanța metrologică și validarea NASA rămân sarcini de implementare. Nu se pretinde că un script mic implementează și validează toate modelele.

## 13.2. Atacuri deliberate asupra informației din viitor

Păstrăm prefixul fix și modificăm artificial full-history K, diametrul final, durata, frame-ul final, compoziția post-test și textul publicației despre rezultat. Input hash și predicția numerică trebuie să rămână aceleași la RNG controlat.

Testăm mutarea sliderului înapoi după un update ulterior; trebuie restaurată starea corectă. Testăm promptul SAM ales dintr-un frame viitor și dependența indirectă a măștilor. Testăm un split random-frame; registry-ul trebuie să îl respingă.

Un LLM din motorul de dovezi poate cunoaște rezultatul publicat. Nu avem cum să garantăm că nu îl știe. Garantăm însă că fluxul numeric nu primește decât câmpurile whitelist ale `PrefixBundle`, fără embeddinguri sau features text generate liber.

## 13.3. Metrici pentru traiectorii

Raportăm MAE/RMSE pe run și apoi agregare egală pe grupurile relevante, nu numai media tuturor cadrelor. Pentru diametru folosim m intern și mm la afișare. Pentru contur folosim distanță geometrică în aceeași unitate și separăm bias de scală de eroarea temporală.

Pentru distribuții folosim un proper score relevant, de exemplu log predictive density sau CRPS pentru scalar. Pentru samples X₁,…,X_M, un estimator CRPS este:

\[
\frac1M\sum_m|X_m-y|-\frac1{2M^2}\sum_{m,n}|X_m-X_n|.
\]

Formula este pentru samples cu ponderi egale; cu importance weights se adaptează ambele sume. Comparatorii trebuie să estimeze aceeași cantitate și același tip de bandă. Nu comparăm o bandă latentă cu una measurement-predictive ca și când ar fi identice.

Raportăm interval width împreună cu coverage, nu coverage singur: o bandă enormă poate acoperi datele fără valoare practică. Coverage nominal de 95% nu este garantat de numele Bayesian, bootstrap sau conformal.

## 13.4. Metrici pentru evenimente

Un event-time MAE nu poate folosi timpul de cenzurare drept timp de stingere. Evaluarea poate folosi likelihood-ul recordului observat, scoruri de risc cause-specific la orizonturi admise și Brier adaptat cenzurării numai cu ipotezele și estimatorul de cenzurare documentate.

O clasificare a cauzei finale este un comparator pentru altă țintă decât forecast-ul până la h. Nu afirmăm superioritate comparând AUC-ul unui target cu log loss-ul altuia. Pentru ACME menționăm explicit politica operatorului și sesiunea de generalizare.

## 13.5. Metrici pentru metrologie

Măsurarea video este evaluată înainte de dinamică: distanță între contururi de referință și contururi extrase, eroare de scară, eroare de timp, drift, eșecuri de tracking și topologie. Segmentation IoU poate fi un diagnostic, dar nu înlocuiește eroarea în mm a cantității care intră în model.

Referințele sunt independente de ajustarea modelului. Dacă nu există referință fizică suficientă, raportăm acordul cu anotatorul și sensibilitatea calibrării separat; nu transformăm acordul pixelilor într-o precizie absolută în metri.

## 13.6. Inferență și convergență

Pentru MCMC: rank-normalized R-hat, bulk/tail ESS, divergences, energie și stabilitatea estimanzilor predictivi. Praguri inginerești inițiale precum R-hat≤1,01 și ESS adecvat pentru cuantile sunt alarme utile, nu dovezi suficiente. Se examinează și multimodalitatea/simetriile NN și prior predictive.

Pentru particles: ESS, greutatea maximă, tails, replici, acord cu referința și rata de refinements. Pentru BASS: ansamblul este empiric și dependent de bootstrap, nu 5 draws perfecte dintr-un posterior de rețele.

Dacă datele nu identifică hazardul de disruption sau o configurație ACME, nu publicăm o curbă sigură doar ca să umplem trei carduri din UI.

## 13.7. Statistica pe grupuri și limitele datelor mici

Bootstrap-ul metricilor se face pe unitatea de independență a afirmației: zi/sesiune/specimen, nu frame. Puține grupuri produc incertitudine mare. Raportăm și rezultatele pe caz, fără a le transforma într-un clasament fals de precizie.

Conformal inference cere grupuri de calibrare și exchangeability relevante. Condiția minimă de index pentru un cuantile empiric finit la 95% în split conformal este 19 grupuri, dar aceasta nu dovedește exchangeability, width util sau transfer la altă gravitație. Nu rezervăm artificial cadre drept grupuri pentru a atinge numărul. [L25, L35]

## 13.8. Ce înghețăm înainte de test

Cohorta, targetul, splitul, transformările, regula de admission, baseline-urile, modelul, hiperparametrii, priors, horizon-ul, metricile, policy-ul VOI și alegerea demo-ului sunt înghețate. Predicțiile sunt salvate înainte de reveal.

O corecție ghidată de rezultatele testului îl transformă în dezvoltare. Nu restabilim independența prin redenumirea fișierului sau schimbarea seed-ului. Un test de software detectat fără folosirea outcome-urilor poate fi reparat, dar trebuie documentat dacă schimbă predicțiile și ce informație a motivat reparația.

## 13.9. Gate-uri de release

| Capacitate | Condiție de activare |
|---|---|
| Replay și dovezi | Identitate, unități și surse corecte |
| Reconstrucție istorică | Fit verificat, etichetat ca fit, nu forecast |
| Forecast temporal | Test independent și comparație onestă cu baseline |
| Probabilități de eveniment | Endpoint/cenzurare/protocol calificate și evaluare adecvată |
| Operator video | Metrologie calificată și forecast pe run-uri nefolosite |
| Accelerator neural | Acord cu referința și câștig end-to-end |
| Selecție de observații | Comparație equal-budget și câștig empiric raportat, inclusiv eșecuri |
| Explicație LLM | Numerele și sursele corespund artefactului, fără extindere de domeniu |

Un gate neîndeplinit dezactivează afirmația specifică, nu întregul explorer. Nu înlocuim predicția cu cifre plauzibile generate.

---

<a id="chapter-14"></a>

# 14. Stack-ul performant și granițele lui

## 14.1. Alegerea finală

| Strat | Implementare | Responsabilitate |
|---|---|---|
| Aplicație | Rust + Axum + Tokio + Tower | API, planuri typed, admission, streaming, limite |
| Numeric runtime | Rust + Rayon, arrays contigue f64 | Likelihood/integrale/particles/quantiles/VOI înghețate |
| Training și referință | Python + JAX/XLA + NumPyro | Autodiff, fitting Bayesian, conditioning de referință |
| Rețele științifice | JAX + Equinox; Diffrax unde este necesar | Rezidualele, encoder/flow și operatorul |
| Vision | PyTorch + SAM2.1 + OpenCV + FFmpeg | Extragere și adaptare selectivă |
| Date | Parquet pentru arhivă; Arrow IPC pentru arrays mari | Proveniență, versiuni, transfer eficient |
| Catalog | SQLite / rusqlite + FTS5 | Metadate, findings, căutare lexicală |
| Dense | Vectors f32 normalizați + SIMD | Scoring exact la dimensiunea corpusului curat |
| Retrieval neural | Qwen3 0.6B embedding/reranker pretrained | Batch/offline; serving GPU numai dacă justificat |
| Cache | Moka + artefacte imuabile | Chei complete și limite de memorie |
| Web | React + TypeScript + Vite + TanStack | Explore/Compare/Lab |
| Grafice | uPlot; ECharts lazy pentru hărți/alte vederi | Timeline, intervale și comparații |
| Packaging | Imagini/lockfiles fixate; local CPU demo | Reproducere fără pod permanent |

Acesta este un stack selectat, nu o măsurătoare că Rust învinge JAX pe orice kernel. JAX compilează calcule native; Python ca limbaj de orchestrare nu înseamnă execuția integralelor în bucle interpretate. [T01–T22]

## 14.2. Runtime-ul numeric

Păstrăm ponderile/posteriorul în memorie. Folosim buffers reutilizabile și structure-of-arrays pentru loturile de particles. Un pool Rayon bounded rulează sarcinile CPU; Tokio rămâne pentru I/O. Nu pornim simultan pool-uri Rayon, BLAS și JAX care suprasubscriu host-ul.

Forecast-ul nu așteaptă un LLM, reranker sau apel Jev. Produce artefactul numeric primul; explicația vine separat. Un worker JAX persistent poate rafina un prefix dificil. Nu repornim procesul și compilarea pentru fiecare slider.

Training-ul nu se rescrie în Rust. Doar traseele frozen inference sunt portate când există o referință de paritate. Propunerea de experiment/model compiler generează scheme, ordinea features, unități și expresii simple de kernel, nu un nou framework general de autodiff.

## 14.3. Model IR și prevenirea divergenței între limbaje

`model.ir.json` declară: variabilele și unitățile, normalizarea, structura layer-elor, tensor ordering, priors/transformări, schema observațională, hazarduri, atomii de stop, support predicates și toleranțe. Generatorul poate emite validatoare și cod repetitiv în Rust/JAX.

Testele independente nu provin toate din același generator. Un bug comun de semn sau scară poate face două implementări identice și ambele greșite. Referința analitică/SciPy și fixtures explicite rămân separate.

Un manifest nou se cere dacă se elimină un feature constant, se schimbă ν, DCT normalization, definiția frontului sau convenția timpului. Nu sunt „detalii UI”.

## 14.4. JAX și precizie

Activăm explicit X64 pentru likelihood/integrale/corecții. JIT pe operații batched, nu pe funcții scalare repetate în Python. Folosim shape buckets, mask-uri sigure și warm cache. Sincronizăm `block_until_ready()` la benchmark și includem transferurile. [T01–T05]

Rețelele de propuneri pot fi antrenate FP32, dar monotonicitatea spline-ului și log-density accuracy trebuie verificate înainte de mixed precision. Nu convertim hazardurile/integralele la BF16 pentru a obține un grafic mai rapid. Quantiles și probabilități se testează la aceeași toleranță ca referința.

Pallas/Mosaic/Triton/CUDA custom sunt extensii după profiling, nu dependențe obligatorii. Verificăm API-urile versiunii efectiv fixate, nu o recomandare generică „cea mai nouă”.

## 14.5. Memorie și joburi

Numărul ponderilor nu determină singur VRAM. Activările video, unrolling-ul temporal, autodiff, batch-ul, Qwen resident și prealocarea JAX pot domina. De aceea adaptăm clip length/batch înainte să concluzionăm că trebuie 96 GB.

Modul training eliberează serving workers GPU. PyTorch/JAX/vLLM au bugete explicite, nu fiecare 90% din placă. NUTS începe pe CPU-ul host dacă pilotul arată ESS/s și cost mai bune. Un GPU cu multă VRAM nu garantează FP64 eficient.

Podul Runpod este deja container. Nu presupunem Docker daemon în interior. Construim/fixăm imaginea în mediul adecvat; Docker Compose descrie deployment pe o gazdă care îl suportă, nu o obligație Docker-in-Docker.

## 14.6. Stocare și căutare

Catalogul scientific release este read-only; starea analizelor este separată. Nu folosim SQLite WAL ca bază multi-writer între patru host-uri pe un filesystem de rețea. Fiecare job scrie într-un director unic și integrează prin manifest.

Embeddings sunt calculate în loturi. La corpus mic, scoring exact normalizat este simplu și nu cere vector DB separat. Dacă volumul crește, alegerea ANN se bazează pe recall/memory/latency măsurate. Nu există un principiu că exact scan este mereu cel mai rapid. [T11–T17]

În profilul de 100 USD, Qwen poate fi calculat batch și folosit prin cale CPU/cached. Reranking-ul are timeout și fallback declarat la RRF. Nu ținem GPU închiriat numai ca fiecare căutare din demo să fie instantanee.

## 14.7. Browser

Crosshair-ul și playback-ul se actualizează în instanța graficului, nu prin rerender React global la fiecare pixel. Typed arrays și Web Workers gestionează transformări mai mari. JSON pentru metadata mici, Arrow IPC pentru arrays mari; nu forțăm Arrow pentru un obiect de 1 KB.

Decimarea este numai pentru afișare. Calculele și exportul folosesc datele admise originale. Cutoff-ul se mișcă imediat; cererile sunt debounced/cancelate și rezultatele stale sunt respinse prin versiunea inputului.

Nu adăugăm WebGPU/WASM doar ca să existe încă o tehnologie. Inference în browser ar fi un al treilea runtime de verificat. CPU server/local și overlay Canvas sunt suficiente pentru primul release.

## 14.8. Profilare și criterii

Măsurăm cold load, compilare, warm execution, peak RAM/VRAM, timpul per gradient, ESS/s, throughput metrologie, timpul complet de forecast și VOI, retrieval recall/latency, p50/p95 în UI și rata de refinements.

Ținte de produs, nu rezultate: interacțiuni locale sub 100 ms; rezultat cached rapid; forecast obișnuit warmed sub o secundă unde posibil; cazurile dificile într-un job cu progres clar. Dacă nu atingem ținta, nu ascundem uncertainty sau reducem particulele până devin incorecte.

Nu avem Kubernetes, queue distribuit, vector DB obligatoriu, LLM în bucla numerică sau generator photorealistic de flăcări. Performanța înseamnă eliminarea lucrului redundant și latență previzibilă la aceeași precizie.

---

<a id="chapter-15"></a>

# 15. Bugetul canonic de 100 USD și disciplina Runpod

## 15.1. Ce acoperă plafonul

Plafonul este **100 USD total incremental** pentru poduri, stocare, eventuale servicii externe suplimentare, taxe și comisioane. Abonamentele AI deja plătite sunt separate. Ele nu implică automat credite API. Un Jev/API/hosting neacoperit trebuie să intre în acest plafon sau să rămână dezactivat.

Nu există în acest document un pod rezervat, stoc confirmat în cont sau o factură garantată. Prețurile de mai jos sunt ipotezele planului bugetar precedent, păstrate ca limită de ofertă acceptabilă. Se reconfirmă la checkout. Dacă tariful este mai mare, reducerea orelor sau realocarea este calculată înainte de pornire, nu după depășirea bugetului. [R01–R04]

## 15.2. Planul de bază

| Resursă | Cantitate și tarif de planificare | Alocare USD |
|---|---|---:|
| RTX 3090 24 GB, pod principal | 120 ore facturate ×0,50 USD/h | 60,00 |
| RTX 4090 24 GB, burst opțional | 8 ore ×0,89 USD/h | 7,12 |
| Stocare | 100 GB network volume /14 zile +60 GB scratch în ferestre active | 5,00 plafon |
| **Servicii planificate înainte de taxe** | | **72,12** |
| **Rezervă protejată** | Taxe, comisioane, reluări, API suplimentar | **27,88** |
| **Total maxim** | | **100,00** |

Configurația publicată anterior pentru 3090 indica 16 vCPU și 125 GB RAM. Acestea sunt proprietățile ofertei verificate atunci, nu garanția fiecărui host 3090. CPU model, RAM, disk și contendența se verifică efectiv la pilot. vCPU nu este automat un nucleu fizic exclusiv.

CPU-ul gazdei este deja inclus în prețul podului. Nu îl facturăm a doua oară în calcule. În schimb, nu presupunem concurență CPU/GPU fără pierdere de throughput. Data loader-ul, memoria și I/O-ul sunt partajate.

## 15.3. Aritmetica stocării

Cu tarif Standard de referință 0,07 USD/GB/lună și lună de 30 zile:

\[
100\cdot0.07\cdot14/30=3.2667\text{ USD}.
\]

Cu scratch 60 GB, tarif de referință 0,10 USD/GB/lună și 128 ore agregate de pod:

\[
60\cdot0.10\cdot128/720=1.0667\text{ USD}.
\]

Total orientativ 4,33 USD, rotunjit la 5. Facturarea și dimensiunile reale controlează suma. O perioadă mai lungă de retenție trebuie inclusă; un volum fără pod continuă să coste. Confirmăm compatibilitatea network volume/cloud/datacenter înainte să îl creăm.

Prețurile minime Community menționate anterior, 0,22/0,34 USD/h, ar reduce aritmetic același plan la 34,12 USD cu plafonul de stocare. Nu depindem de găsirea acelei oferte. Nu presupunem aceeași gazdă CPU/RAM, fiabilitate sau disponibilitate. Economiile rămân rezervă. [R02](#source-r02)

## 15.4. De ce nu RTX PRO 6000

Modelele fizice sunt mici și nu cer 96 GB pentru ponderi. Domină integrarea, sampling-ul și activările video. Alegem un working set potrivit pentru 24 GB și măsurăm. Un batch mai mic poate economisi bani fără să modifice modelul, cu ajustarea corectă a optimizării.

RTX 4090 merită numai pentru jobul măsurat. La 0,89 versus 0,50 USD/h trebuie să fie peste 1,78× mai rapid pentru același rezultat GPU ca să reducă acel cost. Comparația include transfer, startup și eventuala muncă CPU simultană pierdută, nu doar throughput-ul unei matrice.

## 15.5. Timpul bugetat nu este timpul dovedit

| Activitate | Interval preliminar, ore GPU |
|---|---:|
| Pilot, compile, I/O | 2–4 |
| Metrologie pretrained | 4–10 |
| Adaptare SAM2 dacă este justificată | 0–6 |
| BASS și verificări | 6–12 |
| Taskuri statistice și primul accelerator justificat | 4–10 |
| VOI/inferență/paritate | 2–6 |
| Embeddings/ranking batch | 1–2 |
| **Total orientativ** | **19–50** |

Pentru fitting Bayesian și referințe rezervăm provizoriu 40–100 ore cumulate de joburi CPU. Aceste intervale nu sunt benchmarkuri, limite superioare sau distribuții de probabilitate. Nu se transferă automat timpii estimați pentru RTX PRO la 3090.

Cele 120 de ore ale podului includ compile, încărcare, transferuri, joburi CPU și perioade active fără GPU. Sunt o limită financiară rezervată, nu demonstrația că tot designul va termina. Suprapunerea se măsoară. Ca organizare, o campanie de 4–6 zile după pregătirea datelor este ținta inițială, nu un termen promis pentru întreaga dezvoltare.

Pentru un fit NUTS:

\[
T_{fit}\approx T_{compile}+\frac{C}{P}(W+S)\bar L\,t_{gradient}+T_{IO},
\]

unde P este numărul de lanțuri efectiv paralele, nu numărul cerut în API. Dacă C=4,P=4,W+S=2500, L̄=128 și t_gradient=0,04 s, termenul numeric este 3,56 h. Acest exemplu ilustrează formula; nu afirmă că modelul sau host-ul va realiza acele valori.

## 15.6. Persistență și oprire

Container disk este scratch și nu trebuie să fie singura copie a checkpoint-ului. Volume disk și network volume au cicluri de viață diferite. Un stop poate păstra anumite volume și le poate factura; terminate poate șterge volumul local al podului. Verificăm copia persistentă și o copie locală cu hash înainte de terminate. [R03–R04]

Nu folosim sold zero ca mecanism automat de oprire: poate pune în pericol datele persistente. Un script local cu timeout nu garantează că facturarea podului s-a oprit; starea providerului trebuie confirmată. Automatizarea care oprește/șterge cloud resources se implementează numai cu permisiune și după backup verificat.

Checkpoint NN la final de epocă și la intervale bounded; sampler state numai printr-un mecanism de reluare testat. Nu concatenăm fragmente de MCMC ca și când warmup-ul și staționaritatea ar fi rezolvate.

## 15.7. Limite operaționale

La 60 USD servicii cheltuite se face revizie. La 72,12 USD servicii planificate nu se lansează joburi noi fără o realocare explicită în interiorul plafonului total. Nu activăm auto top-up. Păstrăm bani pentru stocare și eventuale taxe.

Dacă forecast-ul bugetului depășește 100 USD, ordinea de reducere este: custom kernels și alte explorări → fine-tuning SAM nejustificat → flows suplimentare → folds/sensibilități opționale. Nu reducem prin ascunderea eșecurilor, renunțarea la testul final, numerică nesigură sau numirea unei inferențe neconvergente „validată”.

Un host CPU deja disponibil echipei poate reduce factura numai după verificarea capacității și autorizării. Planul nu depinde de un server al altui proiect și nu îi afectează resursele fără acord.

---

<a id="chapter-16"></a>

# 16. Execuție paralelă pe module și dependențe, fără împărțire pe persoane

Această secțiune descrie **ce trebuie executat în paralel**, nu cui îi revine fiecare sarcină. Echipa are patru membri și folosește agenți frontier pentru implementare și verificare. Alegerea concretă a agentului sau operatorului pentru un task este lăsată orchestratului de dezvoltare. Un modul este definit prin inputuri, outputuri, interfețe, teste și dependențe — nu printr-un nume de persoană.

## 16.1. Fluxuri de lucru care pot începe simultan

| Flux tehnic | Directoare propuse | Primul livrabil executabil | Dependențe/rezultat final |
|---|---|---|---|
| **Date comune și contracte** | `contracts/`, `data/registry/`, `source_manifest/` | Un experiment real cu identitate, unități, surse și `PrefixBundle` valid | Definește tipurile folosite de experți, runtime, UI și evaluator; schimbările de semantică sunt versionate |
| **FLEX / probabilistic** | `science/flex/`, `science/inference/` | Parser PSI-69, baseline D² și likelihood pe o cohortă verificată | Posterior, forecast din prefix, evenimente, model card și comparație pe validation |
| **ACME / event histories** | `science/acme/` | Cohorte normal/inverse, audit SE/FT, baseline cause-specific | Hazarduri dependente de politica observată și diagnostice de cenzurare |
| **BASS / video / front** | `vision/`, `science/bass/`, `data/bass/` | Clip original, calibrare, contur vizibil și timestamp | Metrologie cauzală, operator și verificări pe specimene independente |
| **Runtime / decizie / integrare** | `runtime/`, `infra/`, `decision/` | API tipat pentru surse, prefix, model și predicție | Runtime Rust, calculatorul VOI, versionare/caching, execuție offline și cost-guard |
| **Scientific Intelligence / UI / test independent** | `evidence/`, `web/`, `evaluation/`, `release/` | Explore/Lab care afișează observație reală cu sursă și cutoff | Compare, citări, experiment replay, test sigilat, raport și demo reproductibil |

**Niciun flux nu așteaptă toate modelele antrenate.** Interfața începe cu date NASA reale și fixture-uri etichetate `synthetic/development` exclusiv pentru contractele încă neimplementate. Schema este înghețată înainte de dezvoltarea divergentă. O modificare a termenului `extinction`, a unității unui câmp sau a definiției unui front necesită un ADR și migrare de schemă.

## 16.2. Orchestrarea agenților frontier

Fiecare flux poate lansa, în medii izolate, un agent de implementare, unul de analiză a surselor și unul de verificare adversarială. Nu presupunem că un singur agent sau un singur LLM este infailibil. Agenții nu partajează liber datele de test ascunse; procesul de evaluare se execută într-un context și un filesystem cu acces separat.

Sunt compatibile mai multe branch-uri simultane, dar nu mai mulți writeri pe același fișier/SQLite/checkpoint. Un integrator tehnic coordonează contractele comune; nu este o repartizare permanentă către o persoană anume. Merge-ul cere teste contractuale și un vertical slice în API.

## 16.3. Contractul de lucru al fiecărui agent

Un task executabil conține: (`objective`, `sources`, `allowed_paths`, `schema_version`, `inputs`, `outputs`, `tests`, `forbidden_shortcuts`, `stop_condition`, `compute_budget`, `release_gate`). Exemple:

- „Verifică un join CIR–GMT cu hash-uri și returnează un `ExperimentRecord`, păstrând conflictul neadjudicat.”
- „Implementează integrala pentru likelihood-ul unui eveniment cu cauză cunoscută și verific-o independent față de cuadratură.”
- „Produce un contur BASS în metri dintr-un clip original calibrat; păstrează locatorul sursei și eroarea de scară.”
- „Execută un `PrefixBundle` fără acces la coloane terminale și compară outputul Rust cu referința JAX.”

Un agent nu poate autoriza cloud spend, schimba politica de suport fizic, dezvălui held-out outcomes sau declara un model calificat doar pentru că loss-ul de train a scăzut.

## 16.4. Review fără autopromovare

Verificările trebuie să falsifice ipoteze: semn/unități, inversa integralei, hazarduri, Student-t conditionals, Jacobiani, leak din viitor, calibrare, lățimea intervalelor, coerența API și citarea surselor. Review independent înseamnă implementare/derivare care nu copiază aceeași funcție supusă testului. Acordul a două modele de limbaj este un semnal slab față de o identitate analitică și de măsurători independente.

## 16.5. Integrarea continuă

Fiecare modul contribuie zilnic un traseu complet: **sursă reală → registru → observație → model/baseline → artefact → reprezentare cu proveniență**. CI verifică schemele, testele negative de leak, unitățile și compatibilitatea numerelor de model. `decisions/`, `model_cards/`, `source_manifest/`, `evaluation/` și `cost_ledger/` păstrează probele.

GPU-ul are un singur lease pentru joburile mari. CPU/parse/UI/retrieval continuă în paralel; nu presupunem patru GPU-uri pentru patru persoane. Joburile se lansează doar după un pilot care extrapolează costul și după salvarea checkpoint-urilor în stocare persistentă.

## 16.6. Responsabilități ale echipei ca întreg

Autorizarea cheltuielilor, verificarea regulilor NASA, interpretarea surselor neclare și aprobarea afirmațiilor finale sunt decizii umane comune sau delegate explicit **în afara acestui raport**. Când o etichetă este contradictorie, pipeline-ul păstrează conflictul; când lipsesc metrologia sau suportul, UI arată limita. Nu mascăm aceste cazuri prin valori generate.

---

<a id="chapter-17"></a>

# 17. Sprint final: 7–13 noiembrie 2026, drumul critic și porțile de calitate

## 17.1. Reper temporal și starea reală

**Revizia curentă este datată 7 noiembrie 2026**, seara în România. Evenimentul vizat este **14–15 noiembrie**; în practică rămân aproximativ șase zile de lucru până la debut. Planurile din octombrie și 5 noiembrie sunt istorice, **nu dovezi că taskurile au fost realizate**. La începerea execuției se inspectează repo-ul, datele și checkpoint-urile reale înainte de a marca progresul.

Termenul și regulile finale trebuie reconciliate cu organizatorii; materialele oficiale complete și un eventual nou calendar de resurse nu sunt considerate verificate doar pentru că există un anunț secundar. A se vedea `requirements_receipt`.

## 17.2. Planul operațional pe zile, fără alocări individuale

| Data | Execuție paralelă pe module | Condiție de acceptare |
|---|---|---|
| **7–8 nov.** | Audit al repo-ului, confirmarea activelor NASA, înghețarea schemelor și spliturilor; baseline FLEX; parsare/adjudicare ACME; clip+calibrare BASS; Explore/Lab dintr-un run real; preflight GPU numai când există job pregătit | Un `ExperimentRecord` cu source hash și prefix valid este randat de API/UI; fiecare expert are un target și un baseline măsurabil |
| **9–10 nov.** | Fitting FLEX/ACME pe CPU, metrologie și operator BASS pe GPU; comparări de dezvoltare, referință numerică și scoruri; integrarea Explore/Compare/Lab; primul replay VOI pe FLEX | Cel puțin un forecast demonstrabil din prefix, artefact versionat și traseu de verificare, cu toate erorile/limitele vizibile; plafonul rămâne respectat |
| **11 nov.** | Integrare finală evidence→prediction→decision→reveal; paritate Rust/JAX; validare pe development; examinarea beneficiului flows/SAM finetune | **Freeze al targeturilor, metricilor, spliturilor, modelului candidat și regulii de alegere a demo-ului** înainte de folosirea rezultatului final |
| **12 nov.** | Generare de predicții finale sigilate; deschiderea continuărilor de test doar după semnarea manifestului; evaluarea aggregate, erori și cazuri negative; stabilizare UI | Tabel cu rezultate reale, rate de refuz, incertitudine și comparație cu baseline-urile pe grupuri independente; fără re-tuning pe test |
| **13 nov.** | Verificare licențe și surse, export reproductibil, backup, offline replay, verificarea cerințelor oficiale, repetiție 90/180 s | Release reproducibil pe minimum două calculatoare, inclusiv când Runpod și serviciile LLM nu sunt disponibile |
| **14–15 nov.** | Demonstrație și predare | Fiecare afirmație din prezentare corespunde unui artefact real și unei evaluări raportate |

Acesta este **un program de execuție agresiv**, nu garanția că trei candidați neuronali vor depăși baseline-urile în șase zile. Implementarea cu LLM-uri frontier comprimă scrierea de cod, nu poate inventa calibrare lipsă sau independență statistică.

## 17.3. Drumul critic

\[
\text{definiții fizice} \rightarrow \text{date eligibile} \rightarrow \text{likelihood corect}
\rightarrow \text{fit și validare} \rightarrow \text{inferență calificată}
\rightarrow \text{benchmark de observații}.
\]

Pentru BASS se adaugă calibrarea și front extraction, iar pentru ACME adjudicarea opririlor. UI, evidence engine, evaluarea izolată, schema și packaging-ul avansează în paralel cu aceste dependențe. **Până pe 11 noiembrie trebuie să existe circuitul complet**, chiar dacă unele capabilități neuronale rămân în stadiu de candidat.

## 17.4. Priorități dacă timpul sau bugetul nu ajung

Protejăm: sursele și drepturile, un forecast real pe un prefix, o selecție de observație evaluabilă, separarea testului, dashboardul amplu și demonstrația offline. Următoarele sunt opționale până la justificare măsurată: fine-tuning SAM2, flows suplimentare, numeroase folds, custom kernels, alte modele generative, 3D decorativ.

Păstrăm toate cele trei familii în produs ca dovezi, metrologie și analize, dar **numai predictorii calificați** pot emite forecast numeric ca rezultat valid. Un model care nu bate baseline-ul rămâne comparație sau modul experimental, nu capabilitate declarată superioară.

## 17.5. Registru de riscuri și reacție

| Risc | Semnal | Acțiune |
|---|---|---|
| Calibrare BASS insuficientă | Contur numai în pixeli, fără scară validă | Se schimbă numai cohorta de dezvoltare pe criterii explicite; fără metri fictivi |
| ACME intervenții neidentificate | SE/FT conflict cu narațiunea | Allowed cause/quarantine; numai observed-policy target |
| Posterior neural greu | Divergences, ESS mic, instabilitate | Reparametrizare, baseline sau refuz; nu mai mult compute automat |
| Cost estimat peste plafon | Proiecția joburilor depășește 100 USD | Elimină joburile opționale și validează forecast-ul financiar înainte de lansare |
| Leak din viitor | Tracker, text, cache sau split afectat de sufix | Invalidare experiment, corecție pipeline, test nou corect |
| Model complex fără skill | Baseline egalează sau depășește neural pe validation | Se publică baseline-ul calificat și comparația cinstită |
| Cloud indisponibil | GPU/API indisponibil la demo | Runtime local CPU, surse și artefacte precompute etichetate |
| Cerință oficială nouă | Format, rights sau resurse se schimbă | `requirements_receipt` și release update fără contaminarea testului |

## 17.6. Criteriul de succes

Nu există o arhitectură care garantează câștigarea hackathonului. Ținta este un produs coerent, o contribuție științifică falsificabilă, benchmark independent și o demonstrație care rezistă unei întrebări tehnice. Numărul de agenți, limbajele și VRAM-ul sunt mijloace, nu rezultatul prezentat juriului.

---

<a id="chapter-18"></a>

# 18. Scientific Intelligence: de la întrebare la dovadă și tool

## 18.1. Corpusul și entitățile

Corpusul include publicații, tabele, protocoale și metadate NASA din toate familiile relevante, nu doar cele trei cu predictor. Entitățile sunt `Source`, `Publication`, `Investigation`, `Specimen`, `Run`, `ControlStage`, `Observation`, `Calibration`, `EndpointReview`, `Finding`, `ModelRelease` și `Analysis`.

O publicație nu este un run. Un finding este o afirmație atomică, cu populație, condiții, rezultat, tip de dovadă și locator. O figură digitizată este o observație derivată cu incertitudine, nu o copie perfectă a măsurării originale.

PDF parsing păstrează pagina, tabelul și contextul. Verificarea unei figuri/tabele necesită inspecția imaginii paginii, nu numai textul extras. OCR este ultima soluție pentru pagini scanate, nu metoda implicită pentru un PDF textual. Valorile extrase de un LLM trec prin verificări de unități, intervale și coerență cu sursa.

## 18.2. Retrieval

Pipeline: query frame → filtre tipate → FTS5 lexical + embedding dense → rank fusion → reranking limitat → extindere controlată a surselor → evidence pack → explicație verificată.

Pentru documentele candidate:

\[
\operatorname{RRF}(d)=\sum_{m\in\{lexical,dense\}}\frac{1}{k+r_m(d)},\qquad k=60\text{ inițial}.
\]

k este o alegere de fusion, nu probabilitate sau scor de adevăr. Absența dintr-o listă contribuie zero. SQLite FTS5 are convenția sa de ordine pentru BM25; o mapăm corect la rank. Nu adunăm direct scoruri BM25 și cosine de scări diferite. [T11](#source-t11)

Embeddings normalizate permit cosine prin dot product. Qwen3-Embedding-0.6B și reranker-ul sunt ponderi pretrained cu template și versiune fixate. Instruct/input formatting face parte din experiment. Top30 pentru reranking este o limită inițială de cost, nu un număr universal optim. [T14–T17]

## 18.3. Șase dimensiuni care nu trebuie ascunse într-un singur scor

Relevanță semantică; comparabilitate fizică; fiabilitate experimentală; calitatea sursei; completitudinea dovezii; aplicabilitatea la întrebare.

Un studiu terestru poate fi semantic foarte relevant pentru a explica de ce transferul la microgravitație este limitat. Nu îl eliminăm din dovezi doar pentru că nu poate intra într-un predictor ISS. În schimb, un tool numeric este blocat în afara domeniului său.

Nu inventăm un „fire risk score” comun între picături, materiale și detectoare. Nu transformăm missing metadata într-un scor mic interpretat ca rezultat negativ. Ranking-ul poate prezenta motive și filtre, nu pseudo-precizie.

## 18.4. Planificare agentică bounded

Un planner transformă întrebarea într-un plan mic de tool-uri: găsește surse, citește secțiuni, extrage condiții, compară, solicită calcule admise, apoi redactează. Un verifier verifică claims și numere. Nu avem nevoie de o societate de agenți care votează asupra adevărului la fiecare întrebare.

Tools numerice au scheme stricte. Nici textul planner-ului, nici o decizie Jev nu pot introduce un material nou într-un model. Planner-ul primește statusul `unsupported_domain` și poate continua cu dovezi sau analogii etichetate.

Contextul păstrează întrebarea, scope-ul, sursele deja citite, rezultate de tool și analiza imutabilă. Memoria persistentă poate salva preferințe și istoricul investigației, dar nu rescrie sursele ori checkpoint-urile. Un sumar LLM este un index util, nu noua sursă de adevăr.

## 18.5. Claim contract

Fiecare afirmație consecventă are un tip:

| Tip | Exemplu de structură |
|---|---|
| Observat | Run/canal/timp/valoare și locator |
| Concluzie a autorilor | Pasaj exact + scope-ul experimentului |
| Calcul independent | Input IDs + formulă + unități + rezultat |
| Inferență de model | Analysis ID + versiune + condiționare + incertitudine |
| Mecanism | Referință fizică, cu limita de transfer |
| Ipoteză | Formulare explicită, nu rezultat |
| Lipsă de evidență | Ce lipsește și ce afirmație nu poate fi emisă |

Verificarea unui număr include unitatea, semnul, populația, numitorul, timpul și tipul de bandă. Verificarea unei citări nu se limitează la existența URL-ului: pasajul trebuie să susțină claim-ul. Un link către un raport NASA nu validează automat orice explicație atașată.

## 18.6. Evaluarea motorului de dovezi

Pregătim întrebări factuale, comparații fizice, contradicții de unități, surse insuficiente, extrapolări nepermise și atacuri prin textul documentelor. O parte sunt păstrate pentru evaluare, cu sursele așteptate și explicația de ce sunt relevante.

Metricile: retrieval recall pe setul revizuit, relevanța ranking-ului, precizia citărilor, extragerea numerică/unități, rata afirmațiilor neacoperite, corectitudinea refuzului și latența. Nu declarăm un benchmark de 100% doar pentru că documentele au fost ingestate cu succes.

Când LLM-ul sau reranker-ul este indisponibil, utilizatorul păstrează sursele, filtrele, graficele și rezultatele numerice. Explicația poate fi un template revizuit, etichetat, nu un text generat fals atribuit unui serviciu live.

---

<a id="chapter-19"></a>

# 19. Dashboardul final și demonstrația

## 19.1. Explore

Ecranul principal pornește de la o întrebare și de la findings, nu de la o listă de modele. Arată familiile NASA, tipurile de dovezi, materialele, gravitația, condițiile, variabilele măsurate și limitele acoperirii. Fiecare card are sursa și motivul relevanței.

Exemplu de întrebare: „Ce ne spun experimentele despre influența oxigenului asupra persistenței unei flăcări?” Răspunsul nu amestecă igniție, propagare, stingere vizibilă și cool flame sub o singură etichetă.

## 19.2. Compare

Coloane aliniate pentru fuel/material, geometrie, dimensiuni, gravitație, atmosferă, controluri, calibrare, fază și endpoint. Sistemul explică ce diferă și dacă o comparație este matched, model-conditioned, mecanistică sau doar contextuală.

Scenariile numerice se compară numai în suport comun și cu definiții identice. Dacă presiunea și diluantul sunt confundate, o diferență de coeficient nu devine efect cauzal al CO₂. Condițiile calculate, precum Tad/Zst, nu sunt slider-e fizic independente fără un solver/validator al amestecului.

## 19.3. Experiment Lab

Layout-ul include video/contur sau seria observată, timeline cu cutoff, forecast și uncertainty, condițiile, statusul modelului, panoul de evenimente și drawer-ul de surse/calibrare.

Pentru FLEX: diametre admise, banda latentă și opțional measurement band, probabilitățile A/H/R/U și explicația populației condiționate.

Pentru ACME: incidențe pentru încetare și operator, survival, marcaje de stop programat și eticheta „sub politica experimentală observată”. Nu arătăm o mărime de diametru pe care modelul nu o are.

Pentru BASS: video real, contur măsurat, contur prezis, marcaje de calibrare, quality flags și eroare după reveal. Culorile conturului reprezintă tipul/intervalul, nu temperatură inventată.

## 19.4. Next observation

Acțiunile candidate sunt din prefixul deja elapsed, dar neadmis complet. Panoul arată care măsurătoare ar fi inspectată, costul, utilitatea estimată, eroarea Monte Carlo și motivul eligibilității. Butonul de achiziție nu dezvăluie continuarea viitoare.

După achiziție, se salvează un nou analysis ID. Predicția veche rămâne accesibilă. Putem vedea o schimbare în sens neașteptat; nu animăm mereu o îngustare de bandă. Reveal-ul viitorului este o acțiune separată, controlată de evaluator în benchmark.

## 19.5. Proveniență și reproducere

Analysis drawer arată source/cohort/split hashes, versiunea modelului, data fit-ului, prefixul, unitățile, solver tolerances, seed-ul, statusul suportului, ESS/diagnostics și dacă rezultatul este live, cached sau precomputed.

Exportul include întrebarea, sursele exacte, tabelul condițiilor, calculele, imaginile graficelor, definiția forecast-ului, rezultatele evaluării și limitările. Nu este doar un răspuns LLM cu bibliografie decorativă.

## 19.6. Stări de eroare și accesibilitate

| Situație | Comportament |
|---|---|
| Sursă incompletă | Afișează ce există și ce câmp lipsește |
| Model necalificat | Replay și baseline etichetat; fără forecast promovat |
| Date în afara domeniului | Refuz numeric cu motiv și dovezi apropiate |
| Support mass mare | Afișează U și limitează claim-ul, nu renormalizează |
| Inference dificil | Job bounded, progres/cancel, fără numere fictive |
| Reranker indisponibil | Ordine RRF etichetată |
| LLM indisponibil | Surse și calcule disponibile, explicație template/cached identificată |
| Fără GPU la demo | Numeric CPU și pachet de dovezi local |
| Rezultat stale | Input version mismatch; nu se suprapune peste condițiile noi |

Graficele au tabele accesibile, rezumat textual, unități, stiluri de linie pe lângă culoare și control din tastatură pentru cutoff. Nu condiționăm accesul la date de WebGL sau de un obiect 3D.

## 19.7. Demonstrația de 180 secunde

| Interval | Ce arătăm | Ce dovedește |
|---|---|---|
| 0–25 s | Întrebare, findings ordonate, pasaj NASA | Summarize/rank/interpret și surse reale |
| 25–55 s | Experiment FLEX, prefix și cutoff | Modelul nu a primit continuarea |
| 55–90 s | Forecast și alegerea unei observații ascunse din prefix | Calcul și decizie, nu citirea răspunsului din document |
| 90–120 s | Actualizare înghețată, reveal, baseline și scor agregat | Utilitate testabilă și limite oneste |
| 120–150 s | BASS video+contur sau ACME evenimente, în funcție de calificare | A doua familie fizică, aceeași interfață/proveniență |
| 150–180 s | Refuz de extrapolare nepermisă și export | Încredere, aplicabilitate, reproductibilitate |

Scenariul demonstrativ se alege printr-o regulă de metadata/eligibilitate înghețată, nu după cea mai mică eroare pe test. Cazul poate să nu fie perfect; tocmai de aceea arătăm și rezultatul agregat și cazurile dificile.

Un output precomputed este legitim dacă afișează checkpoint-ul, prefixul și momentul calculului și nu pretinde execuție live. Nu folosim numere de exemplu sau scene generate ca rezultate experimentale.

## 19.8. Ce ar trebui să rețină juriul

„Ignis nu doar rezumă ce a publicat NASA. Poate anticipa un observabil real dintr-un început incomplet, poate alege ce informație merită inspectată și arată dacă această alegere a ajutat, cu toate sursele și limitele la vedere.”

Aceasta este contribuția competitivă urmărită. Nu garantăm unicitatea mondială a metodelor, un clasament la competiție sau o performanță pe care încă nu am măsurat-o.

---

<a id="chapter-20"></a>

# 20. Contracte API, artefacte și reproductibilitate

## 20.1. Obiectele canonice

| Obiect | Câmpuri esențiale |
|---|---|
| `Source` | canonical URL, DOI/PSI/NTRS, version, SHA256, retrieved_at, rights, locator format |
| `ExperimentRecord` | investigation/run/specimen/session, material/fuel, geometry, gravity, controls, cohort/review state |
| `Observation` | quantity, timestamp, original value/unit, SI value, calibration, quality, source locator, available_at |
| `EndpointRecord` | original label, adjudicated allowed causes, interval, intervention, observation state, evidence |
| `PrefixBundle` | model family, cutoff, admitted observation IDs, known controls, support evidence, immutable hash |
| `ModelRelease` | target, domain, config, weights/posterior, priors, observation model, split, numerics, validation, rights |
| `PredictionArtifact` | input hash, model version, curves, probabilities, interval type, units, diagnostics, support, status |
| `AcquisitionAction` | admissible measurement ID, timestamp≤cutoff, cost, quality, not-yet-admitted flag |
| `DecisionTrace` | candidate set, utility/loss, MC settings, chosen action, revealed value, updated analysis, realized score |

## 20.2. Endpoint-uri propuse

`GET /experiments`, `GET /experiments/{id}`, `POST /evidence/search`, `POST /compare`, `GET /models`, `POST /forecast`, `GET /analysis/{id}`, `POST /acquisition`, `POST /brief`.

`POST /forecast` acceptă ID-uri și condiții în schema strictă, nu un text liber cu valorile terminale. Suprapunerea unui parametru manual peste o condiție măsurată produce un `scenario`, nu rescrie experimentul original.

`POST /reveal` este separat și autorizat pentru evaluator/demonstrație. Datele finale pot fi publice în arhiva NASA, dar separarea în aplicație și în pipeline împiedică folosirea lor accidentală. Publicitatea datelor nu face valid un benchmark care le folosește în predicție.

Joburile grele întorc `analysis_id`/`job_id` și stare, nu blochează request-ul nelimitat. Un request duplicat cu aceeași cheie poate primi același artefact; idempotency nu este echivalent cu reutilizarea unui input aproximativ.

## 20.3. Exemplu de rezultat, fără cifre inventate

```json
{
  "analysis_id": "content-addressed-id",
  "model_release": "flex-v3-qualified-revision",
  "input_hash": "sha256-of-prefix-and-settings",
  "status": "completed",
  "prediction_kind": "prefix_forecast",
  "quantity": "liquid_diameter",
  "unit": "m",
  "conditioned_on": ["established_phase", "observations_up_to_cutoff"],
  "arrays_uri": "relative-artifact-path",
  "probability_categories": ["active_supported", "hot_phase_end", "disruption", "unresolved_support"],
  "diagnostics_uri": "relative-diagnostics-path",
  "source_manifest_hash": "...",
  "cached": false
}
```

Schema este ilustrativă. `qualified` nu se setează doar fiindcă fișierul există. Câmpurile pentru valorile numerice sunt produse de modelul efectiv și validatorul verifică shape, unități, finitudine, masă de probabilitate și horizon.

## 20.4. Release directory

```text
release/
  release_manifest.json
  sources_manifest.json
  cohort.parquet
  split.json
  calibration_registry.json
  models/
    flex/{model.ir.json, posterior.*, observation_model.json, model_card.md}
    acme-normal/{...}
    acme-inverse/{...}
    bass/{weights.*, ensemble_manifest.json, model_card.md}
  support_policy.json
  validation/
  evidence.sqlite
  vectors.*
  demo_cases.json
  rights/
  checksums.sha256
```

Secrets nu intră în release. Originalele mari se identifică prin manifest și instrucțiuni de recuperare; bundle-ul trebuie să conțină efectiv activele necesare demonstrației offline, cu drepturile clarificate. Un URL expirat nu este backup.

## 20.5. Model card minimal

Titlu/versiune, problema și populația, observabilele, inputuri interzise, train/validation/test groups, numărul efectiv de run-uri/events/specimene, preprocessing, normalizare, ecuații, priori, code/hardware, metrici, comparatori, eșecuri/refuzuri, cenzurare, uncertainty, domeniu și limitări. Distinge trained-on-real, pretrained și trained-on-model-generated.

Dacă modelul final este baseline-ul, cardul o spune. Dacă o componentă a fost dezactivată din lipsă de evidență, dashboardul păstrează acest status, nu arată un model „coming soon” ca și când ar fi validat.

## 20.6. Cost și observability

Fiecare job salvează start/end, device, node config, CPU/GPU time observat, peak memory, exit state, checkpoints, provider billed duration când este disponibil, tarif și cost estimat/real. Estimat și facturat sunt câmpuri separate.

Trace-ul aplicației păstrează tool calls, input/output hashes, date de calibrare, durata și refuzurile. Nu logăm chei API sau date ale altor proiecte. Dacă o cerere este anulată, jobul numeric verifică cooperativ tokenul de anulare; un request HTTP închis nu garantează că procesul GPU a încetat.

## 20.7. Stări și release gates executabile

Stări: `source_only`, `measurement_qualified`, `fit_only`, `forecast_qualified`, `event_qualified`, `acquisition_evaluated`, `unsupported`, `insufficient_inference`, `failed`.

O stare nu se promovează automat prin simpla încheiere a trainingului. Registry-ul cere artefactele de acceptare. Tipurile de rezultate sunt constrânse astfel încât UI-ul să nu poată prezenta un `fit_only` ca `heldout_forecast`.

---

<a id="chapter-21"></a>

# 21. Literatura: ce am luat în considerare, ce adoptăm și ce nu

## 21.1. Cum se citește această bibliografie

Orizontul inițial al cercetării a fost 28 octombrie 2026. Consolidarea este din 5 noiembrie. Nu folosim o versiune apărută ulterior cutoff-ului ca și când ar fi fost disponibilă la elaborarea inițială. Pentru un build curent poate fi consultată o versiune nouă, dar se marchează separat.

O sursă are trei întrebări distincte: există și este corect identificată? Ce susține metoda și experimentul ei? Există cod/ponderi/date utilizabile cu drepturi compatibile? Un „da” la prima întrebare nu le rezolvă pe celelalte.

Dosarele anterioare au niveluri neuniforme de verificare. Pentru unele surse există review de metode; pentru altele numai abstract, publisher record sau URL furnizat. În această consolidare păstrăm toate pistele importante, **fără să susținem că fiecare lucrare din 2026 a fost recitită integral sau că rezultatele ei au fost reproduse**. Orice verificare automată de acces/titlu este un receipt bibliografic, nu peer review și nu validare a metodei.

Pentru arXiv, identificatorul de bază poate afișa o versiune mai nouă decât cea folosită inițial. Când o metodă controlează implementarea, înregistrați v1/v2/etc., data acelei versiuni și hash-ul manuscrisului consultat. Un titlu accesibil astăzi nu dovedește că toate detaliile acelei versiuni erau disponibile la cutoff.

Lista completă de URL-uri și stări este în registrul [L01–L40], alături de artefactele [T14–T31]. Acest capitol explică decizia tehnică, astfel încât un chat nou să nu reînceapă de la o listă de buzzwords.

## 21.2. Operator learning: ideea utilă și condiția de date

**FNO [L01](#source-l01)**, Li et al., preprint 2020 / ICLR 2021, introduce un operator învățat care combină transformări locale și convoluții parametrizate în spațiul Fourier. Forma schematică este

\[
v_{\ell+1}=\sigma(W_\ell v_\ell+\mathcal F^{-1}(R_\ell\mathcal Fv_\ell)).
\]

Avantajul este o reprezentare eficientă a mapării dintre funcții. Nu rezultă însă că orice imagine oferă starea fizică necesară. Ignis ia ideea de operator pentru conturul BASS efectiv măsurat și folosește o bază neperiodică mică, nu un câmp de temperatură inventat.

**Factorized FNO [L02](#source-l02)** reduce costul operatorilor multidimensionali prin factorizare. Este relevant când există câmpuri 2D/3D și cost spectral semnificativ. Pentru 32 noduri ale unui front 1D, principala dificultate este cohorta și semantica observației, nu tensorizarea unui operator foarte mare.

**DeepONet [L03](#source-l03)** separă codificarea funcției de intrare de evaluarea la coordonate de ieșire. Este o alternativă serioasă pentru query continuu. Nu elimină necesitatea unor perechi input-function/output observabile și suficiente. Nu lansăm un turneu DeepONet/FNO/GraphNO doar pentru a avea mai multe nume în raport.

**Geo-FNO [L04](#source-l04)** și operatorii pe mesh/graph tratează geometrii variabile. Merită dacă cohorta chiar conține variații de geometrie măsurate și o țintă spațială comună. Introducerea lor într-o cohortă mică și o geometrie fixată ar crește costul și numărul de ipoteze fără avantaj demonstrat.

## 21.3. Lucrările recente despre observații parțiale

**Fundamental Physics Knowledge [L05](#source-l05), arXiv:2602.15184.** Review-ul anterior descrie combinarea traiectoriilor fizice mai simple cu sarcina țintă. Ideea utilă este pretraining/auxiliary learning pe componente ieftine, nu concluzia că putem elimina chimia și obține combustie reală. Exemplul de transfer discutat în dosarul anterior privește smoke/ScalarFlow; nu este dovadă că un surrogate de fum înțelege picături de metanol.

Cod și checkpoint-uri au fost identificate anterior în organizația autorilor. Nu le-am executat pentru Ignis. Transferul la un observabil NASA cere un studiu nou. Prin urmare importăm principiul de învățare etapizată, nu adăugăm un simulator nevalidat ca sursă de adevăr.

**LANO [L06](#source-l06), arXiv:2601.15547.** Masked observations și latent autoregression sunt relevante pentru lipsurile din măsurători și reconstruirea unei stări. Lecția aplicabilă este că masca și observation operator trebuie definite precis. A masca o temperatură măsurată nu este aceeași problemă cu a deduce temperatura din intensitatea RGB. Folosim causal masking și taskuri de prefix; nu pretindem un latent termochimic identificat.

**Neural Operator Processes [L07](#source-l07), arXiv:2606.22946.** Contextul de observații condiționează o distribuție asupra funcției/operatorului. Este apropiat conceptual de experiența „adaug o măsurătoare și se schimbă forecast-ul”. Dar o arhitectură probabilistică nu garantează coverage în alte rezoluții sau domenii. Păstrăm likelihood-ul explicit și corecția de importanță în loc să luăm banda unei rețele ca adevăr.

**Operator Splitting [L10](#source-l10)** și **Function-space architectures [L11](#source-l11)** au fost furnizate ca piste recente. Identitatea/metodele și artefactele nu sunt suficient stabilite în materialul recuperat pentru a controla designul. Sunt păstrate cu URL, nu prezentate ca inovații implementate sau dovezi ale superiorității Ignis.

## 21.4. Combustion-specific neural methods

**Koopman-inspired intrinsic flame instabilities [L08](#source-l08).** Ridicarea într-un spațiu latent cu evoluție mai aproape de liniar și obiective multi-step sunt idei utile. Sarcinile de instabilitate a fronturilor premixed nu sunt automat picături diffusion-flame sau fronturi luminoase pe material solid. Nu transferăm un checkpoint doar pentru că titlul conține „flame”.

**RFNO-PINN [L09](#source-l09).** Constrângerile fizice pentru multispecies pot ajuta când stările, reacțiile, condițiile la limită și datele relevante sunt disponibile. În nucleul nostru, specii și temperaturi nu sunt etichete observate. O funcție de pierdere PDE nu compensează un inverse problem neidentificat. Referința rămâne neobligatorie până la verificare completă și un caz compatibil.

**F-FNO pentru turbulent jet reconstruction [L13](#source-l13).** Review-ul anterior descrie reconstrucția temperaturii/speciilor din informație fizică despre OH. Aceasta este o observație mult mai specifică decât o imagine color. Nu confundăm concentrația OH, chemiluminescența OH* și un contur vizibil. Aplicabilitate posibilă numai pentru instrumente și calibrare matched.

**Biomass pyrolysis gas surrogate [L14](#source-l14).** „Gaz de piroliză” nu înseamnă că modelul învață o interfață solid-gas în mișcare sau că rezolvă piroliza probei BASS. Este inspirație pentru compoziții și surrogate-uri, nu dovada că un solver/surrogate este pregătit pentru NASA.

**Hydrogen flame reaction-rate CNN [L15](#source-l15), arXiv:2609.32552.** Lecția metodologică din review este suficiența observabilelor: câmpul/variabila de progres și contextul spațial determină ce poate fi inferat. Aceasta susține analiza identifiabilității, nu ambiția de a produce un câmp chimic din diametru.

**ChemKANs [L32](#source-l32)** și **Extended FNO for stiff kinetics [L33](#source-l33)** tratează aproximarea chimiei și stiffness. Ar fi relevante pentru accelerarea unui mecanism numeric validat, nu pentru a înlocui auditul observațiilor. Nu avem nevoie de ele pentru cele trei ținte actuale.

## 21.5. Dinamici continue și hibrid mecanism–rețea

**Universal Differential Equations [L21](#source-l21), arXiv:2001.04385**, Rackauckas et al., este precedentul conceptual pentru a păstra o structură fizică și a învăța un rezidual. Nu există o regulă că orice rezidual NN devine fizic identificabil. De aceea rezidualul FLEX este mic, ancorat și bounded, iar V1 rămâne baseline.

**Neural CDEs [L20](#source-l20), arXiv:2005.08926**, Kidger et al., oferă un hidden state condus de o cale interpolată a observațiilor. Este un candidat bun pentru date neregulate. Totuși, interpolarea poate introduce informație viitoare dacă este construită pe întreaga serie. Encoderul actual de prefix este mai simplu de auditat și nu cere integrarea încă unui sistem dinamic pentru fiecare propunere.

Latent ODE/SDE, state-space și Koopman pot fi studiate ca alternative după un rezultat negativ diagnosticat. Nu sunt adăugate toate simultan. Fiecare latent trebuie interpretat ca reprezentare statistică până când există dovezi că reprezintă o temperatură, compoziție sau energie.

## 21.6. Amortized inference și densități

**Neural Spline Flows [L22](#source-l22), arXiv:1906.04032**, Durkan et al., oferă transformări monotone cu inverse și Jacobian evaluabile. Este potrivirea directă pentru dimensiunile locale 2–12 ale Ignis. Rețeaua propune local params; densitatea sa permite corecția exact formulată în capitolul 10.

**Automatic Posterior Transformation [L23](#source-l23), arXiv:1905.07488**, Greenberg et al., este relevant pentru training de posterior din sarcini simulate și gestionarea distribuției de propunere. Ignis are likelihood explicit, deci nu pretinde că trebuie să facă likelihood-free inference. Folosește sarcinile generate pentru accelerare și păstrează verificarea distribuției-țintă.

**Flow Matching [L24](#source-l24), arXiv:2210.02747**, Lipman et al., construiește transport generativ prin câmpuri de vectori. Este puternic în alte domenii, dar sampling-ul prin ODE și evaluarea densității pot fi costuri inutile pentru un posterior mic cu spline flows. Nu îl alegem doar pentru recență.

**Student-t Processes [L36](#source-l36), arXiv:1402.4306**, Shah, Wilson și Ghahramani, justifică tratarea coerentă a marginilor și condiționalelor, nu inovații Student-t independente care ar pretinde același proces continuu. Construcția cu scară latentă și algebra OU sunt explicate în raport.

**NUTS [L37](#source-l37), arXiv:1111.4246**, Hoffman și Gelman, este metoda de sampling, nu un certificat automat de convergență. Dimensiunea mică nu elimină corelațiile puternice, simetriile NN sau costul gradientului.

**Joint longitudinal/event prediction [L34](#source-l34), arXiv:1303.2797**, este precedentul legării traiectoriilor de distribuția evenimentului. Aplicarea în Ignis trebuie să respecte endpoint-urile și censoring-ul din combustie, nu să copieze mecanic un model clinic.

## 21.7. Vision și reconstrucție

**SAM2 [L17](#source-l17)** este backbone-ul selectat deoarece există implementare și ponderi identificate. Modelul poate segmenta obiecte în video; metrologia cere suplimentar calibrare, boundary definition și validare. Ponderile nu au fost antrenate de la zero pentru Ignis.

**SAM2 microgravity droplet measurement [L16](#source-l16), arXiv:2607.16587**, este o pistă de aplicație apropiată. Valorile raportate în abstractul citat anterior nu se transferă ca eroare a sistemului nostru. Manual agreement, număr de imagini și incertitudine fizică sunt mărimi diferite. Se verifică independența run-urilor și calibrarea înainte de folosirea unui rezultat comparativ.

**Gaussians on Fire [L18](#source-l18), arXiv:2511.22459**, este reconstrucție de aspect/radianță din camere potrivite, nu predicție de incendiu într-o altă condiție și nu câmp de temperatură certificat. Multiview/calibrare și licențele componentelor contează. O scenă impresionantă nu este o prioritate în bugetul de 100 USD.

**Sparse-data diffusion reconstruction [L27](#source-l27), arXiv:2510.19971**, arată valoarea condiționării pe observații fizice, dar nu rezolvă lipsa acelor observații. Este păstrată ca metodă comparativă pentru un eventual dataset cu câmpuri reale, nu ca generator de termografie din RGB.

SAM3 [T31](#source-t31) a fost discutat ca opțiune mai nouă. Nu îl alegem fără avantaj metrologic și drepturi/compatibilitate clarificate. Recența nu este un criteriu suficient.

## 21.8. Foundation models și representation learning

**Poseidon [L29](#source-l29), arXiv:2405.19101**, are artefacte identificate anterior pentru PDE fields. Intrările/ieșirile și licența ponderilor trebuie potrivite cu problema. Nu umplem patru canale de fluid cu un singur diametru ca să folosim un checkpoint.

**PDE-FM [L12](#source-l12), arXiv:2511.21861**, este în review-ul anterior o contribuție de workshop AI2ASE, nu automat AAAI main conference. Statusul, codul și ponderile trebuie verificate distinct. Un encoder Mamba/FNO multimodal nu oferă gratuit stările termochimice absente.

**Walrus [L28](#source-l28), arXiv:2511.15684**, **PDE-JEPA [L30](#source-l30), arXiv:2609.34715**, și **Phys-JEPA [L31](#source-l31), arXiv:2606.16076** sunt piste pentru reprezentări fizice. Nivelul de verificare diferă; nu promitem checkpoint-uri de combustie. Un latent JEPA predictiv poate fi util fără interpretabilitate de stare, dar trebuie testat pe aceeași sarcină și aceleași grupuri înainte să înlocuiască structura simplă.

**Chronos-2 [T29](#source-t29)** este un posibil comparator pretrained de serie temporală, nu un model care știe implicit phase endings, cenzurare, unități sau limite de suport. Într-un buget limitat este opțional, nu un al cincilea program de cercetare.

**MOTION [T30](#source-t30)** a apărut în lista de piste. Licența identificată anterior nu dovedește contribuția sau compatibilitatea metodei. Nu are rol obligatoriu în release.

## 21.9. Decizii, calibrări și identificare

**Loss-Driven Bayesian Active Learning [L19](#source-l19)** motivează legarea acquisition de loss-ul relevant. Utilitatea Ignis este derivată explicit; nu împrumutăm automat un gain publicat pe alt benchmark. Succesul se măsoară pe observațiile reale și viitorul sigilat.

**Hierarchical conformal inference [L25](#source-l25)** și **conformal overview [L35](#source-l35)** arată de ce grupurile și exchangeability contează. Un split random-frame cu mii de cadre nu oferă mii de exemple independente de calibrare.

**Weak SINDy [L26](#source-l26)** reduce sensibilitatea la diferențierea zgomotoasă prin forme slabe. Este un comparator/diagnostic interesant pentru relații reduse, dar cere state-uri observabile și o bibliotecă identificabilă. Descoperirea unui termen într-un fit nu îl face lege de combustie.

**Bayesian PMMA parameters [L38](#source-l38)** și studiile de limite PMMA [L39–L40] sunt contexte fizice. Istoria termică, gravitația, geometria și definiția limitei influențează comparabilitatea. Un posterior ajustat la centrifugă ori 1 g nu se transferă ca măsurare orbitală.

## 21.10. Decizia de literatură în forma cea mai scurtă

| Familie | Importăm | Nu presupunem |
|---|---|---|
| Neural operators | Mixing spațial și condiționare pe geometrie observată | Acces la câmpuri termochimice neobservate |
| Neural differential models | Rezidual bounded în jurul unei reduceri fizice | Identificarea mecanismului doar din flexibilitatea NN |
| Neural posterior estimation | Propuneri rapide cu densitate evaluabilă | Corectitudine fără reponderare/diagnostice |
| Pretrained video | Măști și tracking care reduc efortul | Calibrare absolută sau lipsa leakage-ului |
| Bayesian decision theory | Utilitate pentru o întrebare precisă | Siguranță operațională certificată |
| Foundation models | Comparatoare și transfer dacă canalele coincid | „Generalist” înseamnă toate regimurile NASA |

Nicio lucrare, indiferent cât de recentă, nu înlocuiește un experiment de evaluare pe ținta noastră. Acesta este motivul pentru care designul este modern și ambițios la nivel de circuit științific, dar nu umflă arbitrar rețelele.

---

<a id="chapter-22"></a>

# 22. Decizia arhitecturală finală și instrucțiuni pentru continuare

## 22.1. ADR-IGNIS-V3-100

**Decizie:** construim un laborator evidence-linked, cu experți tipați FLEX/ACME/BASS, metrologie calibrată, inferență probabilistică și selecție de observații, într-un dashboard comun. Păstrăm modelarea V2 unde este susținută, dar limităm compute-ul la 100 USD și activăm optimizările numai după profiling/validare.

**Date:** NASA și publicații primare cu sursă/rights/identitate. FLEX este primul traseu dinamic bine delimitat; ACME oferă event-history sub politica observată; BASS oferă ținta de geometrie vizibilă numai după calificarea subsetului. Restul familiilor rămân în evidence product.

**Matematică:** likelihood normalizat, model de observație Student-t coerent, constrained FLEX rate, competing causes și support mass, ACME policy/stop atoms, operator BASS geometric neperiodic. Niciun latent neobservat nu este numit temperatură sau chimie măsurată.

**Inferență:** posterior/reference corect întâi; ACME nou-session pornește cu cuadratură bidimensională verificată, nu cu două flows obligatorii; flow optional pentru câștig de latență, cu density correction și diagnostic. Folosim JAX pentru training și Rust pentru runtime frozen; paritatea nu înlocuiește verificarea independentă.

**Decizie:** VOI inițial pe FLEX, apoi BASS dacă observation model permite; nu inventăm acțiuni pentru ACME event-only; măsurătorile candidate sunt nerevelate din trecutul admis; Jev opțional pentru semantic routing. Nu RL obligatoriu, nu recomandări autonome pentru siguranța unei nave.

**Cost:** 72,12 USD servicii planificate +27,88 USD rezervă =100 USD. Nu RTX PRO permanent. Nu presupunem credite API din abonamente.

**Calificare:** baseline, grouped split, test sigilat, intervale și eșecuri. Un model care nu demonstrează valoare nu este promovat prin complexitate sau printr-o animație.

## 22.2. Ce rămâne încă de stabilit din dovezi

1. Textul complet, ediția și resursele oficiale ale challenge-ului, calendarul/formatul și regulile de lucru anticipat.
2. Cohortele efective, identitățile și drepturile fișierelor la versiunea folosită.
3. Mapping/calibrare și boundary definition pentru BASS; suficiența unui subset planar.
4. Semantica timpilor și intervențiilor ACME, inclusiv conflictele etichetă/narațiune.
5. Proprietățile reale ale host-ului și tariful/stock la checkout.
6. Costul măsurat al fitting-ului, metrologiei și inferenței; convergența posteriorului.
7. Skill-ul pe experimente independente; coverage și valoarea achiziției.
8. Avantajul efectiv al fine-tuning-ului SAM, al flow-urilor și al Jev.
9. Identitatea și metodele tuturor pistelor bibliografice recente încă neconfirmate.

Acestea nu sunt „detalii rezolvate de agenți fără verificare”. Sunt dependențe concrete care au owner și criteriu de acceptare.

## 22.3. Ce nu se poate afirma încă

Nu există în acest raport o dovadă că Ignis prezice mai bine decât Fotia/MicroFire; că BASS va avea suficiente măsurători; că toate modele vor converge în orele estimate; că probabilitățile sunt calibrate pe NASA; că Jev este necesar; că resursele oficiale au fost publicate la o anumită dată; că un pod este disponibil la un tarif; sau că echipa va câștiga.

Există o arhitectură selectată, un program experimental definit, surse și limitări documentate, un buget cu plafon și verificări matematice executabile. Acestea justifică implementarea, nu preanunță rezultatul.

## 22.4. Primele acțiuni pentru echipă

Inventariați codul/datele care există efectiv. Fixați schemele și spliturile. Produceți un vertical slice pe o observație reală. Implementați likelihood-urile și cazurile analitice. Pregătiți datele și joburile local. Autorizați un pilot limitat, măsurați, apoi programați campania în plafon. Păstrați din prima zi dashboardul și evaluatorul conectate la aceleași contracte.

## 22.5. Instrucțiunea de handoff pentru un chat nou

> Acesta este documentul canonic al proiectului IGNIS, V3.1, revizuit 7 noiembrie 2026, cu buget total de 100 USD, echipă de patru, execuție organizată pe module fără atribuiri personale și implementare asistată de agenți frontier. Folosește-l ca sursă de context; nu reveni implicit la V1 sau la podul RTX PRO de 750–950 USD. Citește în primul rând contextul, ADR-ul, starea calificării și capitolul relevant taskului. Distinge datele moștenite, propunerile, sursele neconfirmate și rezultatele reale. Nu presupune că vreun model NASA este antrenat sau calificat doar fiindcă matematica este descrisă. Nu modifica targeturi, unități, faze, splituri ori buget fără revizie explicită. Poți implementa agresiv în paralel, dar păstrează teste independente, proveniență, izolarea continuării și limitele de cost. Înainte de recomandări curente despre hardware, API, bibliografie sau regulile competiției, verifică sursele primare. Nu porni servicii cu cost și nu modifica repository-uri externe fără autorizație separată. Fiecare livrabil trebuie să fie executabil/inspectabil și să aibă criteriu de acceptare. Scopul este un instrument științific convingător și corect, nu complexitate pentru aparență.

## 22.6. Întrebări care nu trebuie repetate

Bugetul este 100 USD total, nu per persoană. Abonamentele existente sunt în afara lui. Sunt patru persoane; acest raport nu le repartizează taskurile. Echipa dorește arhitectură ambițioasă și folosirea agenților frontier, nu o reducere preventivă la un chatbot. Jev este o idee opțională, nu o cerință obligatorie. Produsul nu este limitat la picături, dar fiecare model este limitat la domeniul validat. Training și cloud spend nu au fost autorizate prin simpla solicitare a acestui raport.

## 22.7. Limita documentului

Documentul poate fi suficient pentru reluarea raționamentului și a implementării într-un chat nou. Nu poate înlocui fișierele sursă, cheile de cont, un checkpoint care încă nu există sau rezultatele unei evaluări neefectuate. Acestea trebuie obținute/produse și atașate release-ului cu proveniență. Când starea proiectului se schimbă, actualizați un jurnal scurt de execuție cu artefacte și hash-uri, fără a rescrie retrospectiv ce era cunoscut înainte de test.

---

<a id="chapter-23"></a>

# 23. Configurațiile inițiale complete și politica de suport

Acest capitol concentrează setările necesare implementării. Sunt **configurații inițiale**, nu hiperparametri validați sau timpi măsurați.

## 23.1. Configurații de model

| Parametru | FLEX | ACME | BASS |
|---|---|---|---|
| Populație inițială | Metanol liber flotant, fază hot stabilită | Normal/inverse separate, fază stabilită | Un material și o geometrie planar-calibrată |
| Stare | q=D² | Istoric de evenimente / hazarduri | Frontul luminos f(s,t) |
| Neural correction | 6→16→1 tanh, rezidual diferențiat și bounded | 9→8→2 tanh, hazard correction | Width16, 4 cosine modes, 3 blocks, FiLM8→16→96 |
| Local variables | b₀,b₁,log D₀ | Două frailties per sesiune | 8 front coefficients +4 velocity coefficients |
| Număr nominal | 158 global +3/run | 129 global/config +2/session | 5825 neural/member +parametrii fizici/statistici expliciți |
| Fitting | NUTS full batch FP64 | NUTS full batch FP64 dacă calificat | MAP/AdamW, 5 group bootstrap members |
| Baseline-uri | D² population, prefix D², V1 | Constant/piecewise și linear hazard | Persistence și viteză constantă |
| Precizie serving | FP64 scientific path | FP64 scientific path | FP64 solver/likelihood; NN precision prin paritate |

Observația FLEX are ν=4, kernel exponential+diagonal nugget și scale pozitive. Priori inițiali largi pentru log(σ_c/1 m), log(σ_n/1 m) pot fi centrați la log(5×10⁻⁵), cu deviație 1,5; aceștia sunt **priori reziduali de proiectare**, nu precizie instrumentală de 50 µm. Separarea σ_c/σ_n cere identificare din cadență și date. Dacă nu este identificată, se fixează o componentă pe bază externă sau se restrânge modelul prin revizie, nu se pretinde separare reușită.

ℓ_obs are unitatea s și un prior larg în log în jurul unei scări de 0,2 s, dacă aceasta trece prior predictive la cadența reală. Priorii sunt revizuiți înainte de final-test și documentați; nimic nu îi transformă în adevăr NASA.

## 23.2. Numerică și toleranțe

Solverul de referință FLEX poate începe cu 32 noduri Gauss–Legendre și până la 48 pași de bisection, comparat cu 64/128 noduri și o referință independentă. Acest traseu nu este cel rapid promis pentru serving. 10000 observații ×32 noduri ×48 evaluări ar însemna 15,36 milioane evaluări de integrand pentru o singură funcție de densitate, înainte de hazarduri și gradient. Algoritmul trebuie optimizat, nu numai portat în Rust.

În coordonata r, dt/dr este negativ. O integrală pozitivă în timp se scrie cu limite inversate și Jacobian pozitiv. De exemplu, hazardul cumulativ până la t(r) este integrala de la r la 1 a λ(t(u))·|dt/du|. Acesta este un test de semn important.

Toleranțele se raportează la o scară fizică explicită și la incertitudinea de observație. Nu folosim aceeași toleranță absolută numerică pentru diametru în m și presiune în Pa. Testele de derivative sunt în interiorul domeniului; discontinuitățile de suport sunt tratate separat, nu ascunse sub `NaN→0`.

## 23.3. Admission și suport comun

Ordinea gate-urilor:

1. **Semantic:** familie, fuel/material, geometrie, gravitație, fază, target și versiune.
2. **Measurement:** identitate, timp, unități, calibrare și missingness.
3. **Condition support:** overlap multivariabil real în training.
4. **Numerical:** model calificat, solver/refinement și diagnostice acceptabile.
5. **Horizon:** mărimi/timpi în suport, cu masa U raportată.

Un exemplu de heuristică inițială este cel puțin trei neighbors compatibili pe cel puțin două date, într-o rază stabilită din distanțele leave-one-date-out de training, cu prag inițial percentile95. Acesta este un mecanism operațional de refuz, **nu probabilitate OOD de 95% și nici garanție de coverage**. Se verifică pe dezvoltare și se îngheață. Pentru contexte cu mai puține date, politica poate fi mai restrictivă, nu „completată” cu cadre ca neighbors independenți.

Faptul că fiecare marginală este în interval nu garantează suportul combinației. Pentru scenarii A/B, ambele trebuie să treacă gate-ul și să aibă overlap-ul necesar contrastului. p și χO₂ nu se schimbă independent de toate celelalte consecințe fizice; interpretarea rămâne model-conditioned, nu efect cauzal universal.

## 23.4. Landmarks și targeturi

Cutoff-urile se definesc în secunde de la intrarea fizic revizuită, nu ca procent din viața finală. Exemple de landmarks de dezvoltare sunt 1, 3 și 5 s, numai când prefixul este admis și se poate confirma faza la acel moment. Acestea sunt setări de testat, nu intervale valabile pentru toate NASA runs.

Mai multe landmarks din același run sunt rezultate dependente. Se păstrează toate în raport, dar intervalele agregate folosesc cluster-ul corect. Confirmarea de activitate la τ trebuie să provină din observații disponibile atunci; nu din faptul că ulterior am citit durata completă.

Pentru VOI, cutoff-ul rămâne fix pe parcursul achiziției din trecut. Setul inițial poate avea observații rare în [0,τ], cu o confirmare independent admisă a fazei la τ. Acțiunea dezvăluie un punct din acel interval fără a modifica în secret timpul prognozei.

## 23.5. Contraste și sensul incertitudinii

Pentru un contrast de populație între scenarii A/B, folosim același draw global Θ pentru ambele și integrăm efectele locale conform distribuției definite:

\[
\Delta(\Theta)=E_b[F_H(T\mid c_A,b,\Theta)]-E_b[F_H(T\mid c_B,b,\Theta)].
\]

Distribuția lui Δ asupra Θ descrie incertitudinea contrastului de populație. Probabilitatea că două experimente viitoare individuale au ordine diferită a finalului este alt estimand: ea include efecte locale și event times diferite. Nu amestecăm cele două în aceeași bandă.

## 23.6. Ratele de refuz fac parte din rezultat

Nu evaluăm numai cazurile la care modelul a răspuns și ignorăm restul. Raportăm coverage-ul de domeniu, masa U, numărul de prefixe respinse și motivele. Pentru validarea predictive, numitorul include toate cazurile admise prospectiv, chiar dacă ulterior apar eșecuri de geometrie sau observation loss. Metricile care nu pot fi calculate pe anumite cazuri au un motiv documentat și o analiză de sensibilitate, nu dispar din tabel.

---

<a id="chapter-24"></a>

# 24. Brief-uri tehnice executabile pe module, fără alocare personală

Brief-urile sunt contracte pentru agenți frontier și workflow-uri CI; **nu sunt repartizate către persoane** și nu presupun că există deja repo-ul, modelele antrenate sau joburile cloud. Un job primește un `budget_cap`, versiunea contractului și acces numai la sursele necesare. Niciun agent nu primește drept de a deschide testul final, de a modifica regulile fizice sau de a crea costuri în cloud fără autorizare.

## 24.1. Date NASA și compilatorul comun de observații

**Misiune:** construiți manifestul imutabil al surselor, reconciliați identitatea run/specimen/session, păstrați valorile brute și corectate, unitățile SI, control histories și versioned cohort/split manifests. **Input:** PSI/NTRS, metadata, tabele, fișiere, DOI și documentație de calibrare. **Output:** `ExperimentRecord`, `Observation`, `EndpointRecord`, `PrefixBundle`, source hashes și flow table. **Acceptare:** reproducere deterministică, lipsă de duplicare a unui run prin documente diferite, review explicit al ambiguităților și imposibilitatea includerii sufixului în prefix.

## 24.2. FLEX — modelul probabilistic al picăturilor

**Misiune:** implementați traseul complet PSI-69 → cohortă calificată → baseline-uri D² → likelihood Student-t → model V1 → rezidual neural bounded → posterior/forecast din prefix. **Input:** test sheets, serii, calibrare, evenimente și split. **Output:** model release, posterior arrays, forecasts, diagnostice și scoruri pe validation. **Acceptare:** identitatea CIR/date/GMT; recuperarea exactă a legii D² în caz limită; gradient/inversă independent; event likelihood și mass accounting; fără K calculat pe întreaga ardere ca input. **Nu impuneți flow** înaintea profilării latenței.

## 24.3. ACME — competing risks și opriri programate

**Misiune:** reconciliați tabelele normal/inverse, origin time, established-phase, evenimente SE/FT, intervenții, pierderi și ambiguități. Construiți două modele separate de hazard sub politica observată, cu baseline-uri. **Input:** PSI-159 + control logs. **Output:** cohort tables, `cause_set`, likelihood including interval/atoms, model cards și curbe. **Acceptare:** probabilitățile însumează unu, conflictul SE/limită termică rămâne vizibil, stările post-test nu ajung în input, terminările operatorului nu sunt prezentate ca extinguishment spontan, iar un eventual estimand contrafactual este dezactivat până la identificare separată.

## 24.4. BASS — metrologie și operatorul spațial

**Misiune:** alegeți prin criterii documentate un singur domeniu material/geometrie, legați clipurile de run-uri și calibrare, extrageți un contur luminos definit și antrenați operatorul numai după ce există ground truth metrologic admis. **Input:** video și timestamp-uri reale, fișe, control stages, scale/perspective. **Output:** calibrated contour sequences, quality flags, causal segmentation, five-member operator candidate, benchmark geometric. **Acceptare:** contur metri verificabil și sursă exactă, tratarea ocluziilor/topologiei, no-future-prompt, cosine normalization, 32/64-grid sensitivity, comparație persistence/constant-velocity pe run-uri nefolosite.

## 24.5. Inference runtime și acceleratoare neuronale opționale

**Misiune:** condiționare pe prefix pentru fiecare expert, cu prior, likelihood, event survival și calibration uncertainty corecte. Pentru ACME testați quadratura Gauss–Hermite 2D cu normalizare joint; pentru celelalte folosiți sampling/refinement controlat. Encoder/flow se antrenează **doar unde benchmarkul justifică** economia de latență. **Acceptare:** ESS, log weights, spline inverse/Jacobian, paritate referință, stabilitate pe seeds; un accelerator nu poate schimba ținta posteriorului.

## 24.6. Active observation și Decision Intelligence

**Misiune:** definiți un target viitor și o loss, construiți un pool de măsurători încă nerevelate dar în prefix, calculați reducerea anticipată a Bayes risk cu cost și comparați cu random/uniform/uncertainty. **Output:** `DecisionTrace`, acțiunea selectată, valoare estimată, măsurătoarea reală dezvăluită și scorul ulterior pe continuarea ascunsă. **Acceptare:** același buget, no-leak, scoring pe grupuri independente și raportarea cazurilor unde alegerea a înrăutățit forecast-ul. **Jev este opțional**, doar pentru intent routing, evaluat separat.

## 24.7. Runtime Rust, API, model IR și cost-guard

**Misiune:** implementați schema comună, gateway-ul Rust/Axum, calculul de servire versionat, cache-ul, planificatorul de joburi GPU și proveniența rezultatelor. **Input:** model manifests, weight/posterior arrays și numeric fixtures. **Output:** `forecast`, `acquisition`, `evidence/search`, `compare`, `reveal`, `brief`, cu artefacte imutabile și hash-uri. **Acceptare:** paritate JAX/Rust, refuz de suport fizic, job budget lease exclusiv, fără auto top-up, backup înainte de terminare și fallback offline CPU.

## 24.8. Scientific Intelligence, dashboard, evaluator și release

**Misiune:** construiți Explore/Compare/Experiment Lab, FTS5+dense retrieval și citările exacte, timeline/cutoff/video/uncertainty și exportul reproducibil. Evaluatorul sigilat este izolat de training. **Input:** sources, finding records, endpoint semantics, model releases, artefacte de forecast. **Output:** UI completă, benchmark cu baseline-uri, model cards, research brief, offline bundle și script 90/180 s. **Acceptare:** surse pentru fiecare număr important, etichetare observed/derived/inferred/precomputed, final predictions hashate înainte de reveal, rate de refuz, accesibilitate și execuție fără API-uri frontier.

## 24.9. Condiția comună de finalizare

Un task este terminat când livrabilul execută, testele trec, datele și versiunile sunt inspectabile, iar integrarea funcționează. O afirmație predictivă necesită suplimentar evaluare independentă și domeniu fizic calificat. „Am generat codul” și „s-a terminat trainingul” sunt stări intermediare, nu definiția succesului.

---

<a id="chapter-25"></a>

# 25. Proveniența acestei consolidări și limitele de verificare

## 25.1. Materialele de intrare

Documentul reunește contextul conversației, dosarul V2, specificațiile matematice/configurațiile asociate, organizarea modulară a execuției și revizia de 100 USD. Pachetul include un receipt al fișierelor găsite efectiv în mediul de lucru, cu mărime și SHA-256. Un fișier absent nu este declarat citit doar pentru că a fost menționat într-un mesaj.

Istoricul V1 a furnizat inventare și review-uri importante, păstrate aici ca rezultate ale auditului anterior. Nu sunt confundate cu un nou crawl complet al arhivei NASA. Nicio factură cloud, model trained pe NASA sau metrică de generalizare nu este produsă de această consolidare.

## 25.2. Corecțiile care controlează acum implementarea

- Bugetul și infrastructura sunt cele de 100 USD, nu propunerile de RTX PRO permanent.
- Splitul planificat este grouped train/validation/test; robustețea mai multor folds rămâne o extensie declarată.
- Observation model este un Student-t cu margini/condiționale coerente, nu un proces AR-t declarat incorect drept invariant la cadență.
- Stopul programat al debitului ACME are atom de probabilitate, nu un hazard continuu inventat.
- Operatorul BASS are normalizare cosine și unități explicite; un basis choice nu dovedește o condiție fizică la margine.
- Utility-ul se referă la o țintă definită și acțiuni trecute încă neinspectate; nu devine control autonom al unui incendiu real.
- ACME local inference începe cu Gauss–Hermite bidimensional și normalizare joint peste global/local; rețelele de propuneri sunt opționale.
- VOI nu este forțat asupra unui expert event-only fără măsurători candidate informative.
- Jev rămâne rutare opțională și versiune fixată numai după verificare; nu calculator de probabilități fizice.
- Pistele bibliografice 2026 insuficient confirmate sunt marcate, nu transformate în argumente de autoritate.
- Calendarul operațional este actualizat la 7 noiembrie; nu presupune că planurile anterioare au fost executate.

## 25.3. Citarea corectă în livrabilele viitoare

Pentru fapte NASA, citați sursa primară și locația exactă. Pentru un rezultat Ignis, citați analysis ID, versiunea și protocolul. Pentru o metodă, citați lucrarea și descrieți ce a fost adaptat. Pentru un tarif, salvați oferta/dată/provider, nu numai un număr din chat.

Nu citați acest document ca sursă primară pentru un fapt fizic dacă el doar rezumă o lucrare. El este sursa proiectului pentru **decizii și specificații**, nu înlocuitorul surselor experimentale.

## 25.4. Ce înseamnă PASS în anexă

PASS înseamnă că o identitate sau o proprietate a unui test sintetic a trecut la valorile și toleranțele folosite. Nu înseamnă că toate inputurile posibile sunt corecte, că modelul se potrivește datelor NASA, că posteriorul a convergent sau că performanța va fi bună. Rezultatele conțin seed, toleranțe și tipul verificării tocmai pentru a evita această confuzie.

Testele executabile din pachet sunt o referință auxiliară. Implementatorii trebuie să adauge teste pentru schema efectivă, datasetul real, metrologie și runtime. Un test care ar fi trecut numai fiindcă două implementări împart același bug trebuie înlocuit cu o referință independentă.

## 25.5. Starea de predare

Starea documentului este **arhitectură selectată și plan verificabil de implementare**. Când echipa începe execuția, adaugă un jurnal distinct: dată, task, artefact/hash, teste, rezultate, cost și decizie. Nu schimbă retrospectiv cutoff-ul sau criteriul de alegere a modelului pentru a face rezultatul să pară anticipat.

---

<a id="chapter-26"></a>

# 26. Verificări sintetice executabile

**Stare a suitei din pachet: PASS.**

Synthetic mathematical and arithmetic checks only; no NASA training, no cloud jobs, no runtime performance benchmark.

| Verificare | Stare | Detaliu |
|---|---|---|
| parameter_inventory | PASS | `{"flex_neural": 129, "acme_neural": 98, "bass_member": 5825, "encoder": 73856, "all_four_proposals": 712916}` |
| constant_D_squared_limit | PASS | `{"absolute_time_error_s": 2.6645352591003757e-15}` |
| implicit_trajectory_gradient | PASS | `{"r": 0.7251830105036342, "implicit": -0.04030473361991029, "finite_difference": -0.04030473361726905, "absolute_error": 2.641241392264959e-12}` |
| known_cause_interval_not_all_causes | PASS | `{"known_cause_probability": 0.2228323739607458, "all_cause_probability": 0.3342485609411188}` |
| support_mass_partition | PASS | `{"max_partition_error": 1.1102230246251565e-16}` |
| scheduled_operator_stop_atom | PASS | `{"spontaneous_before_stop": 0.3007922426039824, "operator_total_including_atom": 0.6992077573960176, "atom_at_stop": 0.5488116360940264, "post_stop_survival": 0.0}` |
| student_t_joint_equals_marginal_plus_conditional | PASS | `{"log_density_identity_error": 3.3306690738754696e-16}` |
| OU_Kalman_equals_dense | PASS | `{"logdet_error": 0.0, "quadratic_error": 0.0}` |
| BASS_cosine_normalization | PASS | `{"weighted_orthogonality_error": 6.661338147750939e-16, "constant_coefficients": [1.0, 8.326672684688674e-17, -5.551115123125783e-17, 1.1102230246251565e-16]}` |
| rational_quadratic_spline_inverse_and_derivative | PASS | `{"inverse_max_error": 2.3869795029440866e-15, "derivative_max_error": 2.826214817730488e-10}` |
| defensive_mixture_importance_correction | PASS | `{"seed": 20261105, "particles": 100000, "posterior_mean_estimate": 0.6238921374273441, "analytic_mean": 0.6236080178173719, "absolute_error": 0.000284119609972211, "ESS": 83581.27173084524}` |
| surprising_observation_can_widen_uncertainty | PASS | `{"prior_variance": 2.0, "conditional_variance_after_surprise": 8.120000000000001, "meaning": "A realized observation need not narrow uncertainty."}` |
| ACME_Gauss_Hermite_joint_global_local_normalization | PASS | `{"event_probability_24_nodes": 0.2186277209982742, "event_probability_48_nodes": 0.2186277209982742, "absolute_refinement_error": 0.0, "posterior_global_draw_weights": [0.6605912787250682, 0.33940872127493177]}` |
| budget_arithmetic | PASS | `{"compute_usd": 67.12, "prorated_storage_estimate_usd": 4.3333, "planned_service_cap_usd": 72.12, "protected_reserve_usd": 27.88, "total_cap_usd": 100.0}` |

Cod: `verification/verify_master_math.py`. Rezultate: `verification/results.json`.

Aceste verificări nu sunt o implementare completă a Ignis și nu demonstrează acuratețe, calibrare sau timp de training pe NASA/Runpod. Toleranțele sunt numai cele ale cazurilor sintetice definite în cod. Metrologia, cohortele, inferența de release și paritatea runtime-ului real necesită teste suplimentare.

---

<a id="chapter-27"></a>

# Registru complet de surse și proveniență

Acest registru păstrează separat titlul/pista, nivelul de verificare și rolul în proiect. Un link nu înseamnă că am citit integral lucrarea, rulat codul, confirmat licența tuturor activelor sau reprodus rezultatul. Referințele din 2026 neconfirmate rămân piste, nu dovezi care controlează arhitectura.

<a id="source-c01"></a>

### C01 — NASA Space Apps — Flame in Freefall, pagina indicată de echipă
Sursă: https://www.spaceappschallenge.org/2026/challenges/flame-in-freefall/

**Statut:** Cerință și URL furnizate; lista completă de resurse nu a fost stabilită în dosarul recuperat.
**Rol:** Controlează brief-ul numai după verificarea ediției.
Data evenimentului 14–15 noiembrie 2026 este termenul furnizat de echipă, nu o certificare independentă în acest raport.

<a id="source-c02"></a>

### C02 — NASA Space Apps — ediția 2026
Sursă: https://www.spaceappschallenge.org/2026/

**Statut:** De reverificat pentru calendar și ediție.
**Rol:** Calendar oficial.


<a id="source-c03"></a>

### C03 — NASA Space Apps — participant FAQ
Sursă: https://www.spaceappschallenge.org/resources/participant-faqs/

**Statut:** De reverificat înainte de predare.
**Rol:** Reguli despre echipă, lucru anticipat, predare și jurizare.


<a id="source-c04"></a>

### C04 — NASA Space Apps — legal
Sursă: https://www.spaceappschallenge.org/legal/

**Statut:** De reverificat pentru termenii aplicabili.
**Rol:** Drepturi și obligații de publicare.


<a id="source-d01"></a>

### D01 — FLEX
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-69

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Inventar moștenit: 274 rânduri sumar, 260 istorii lichid listate; 152 istorii metanol raportate ca descărcate anterior. Nu este cohorta calificată.


<a id="source-d02"></a>

### D02 — BASS-II
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-25

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 129 rânduri master, date/video eterogene; cohortă și calibrare încă de calificat.


<a id="source-d03"></a>

### D03 — BASS
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-26

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 122 rânduri în tabelul anterior; protocol distinct de BASS-II.


<a id="source-d04"></a>

### D04 — ACME Cool Flames Investigation with Gases
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-159

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 179 normal +154 inverse ne-goale; semantica timpilor și intervențiilor trebuie adjudicată.


<a id="source-d05"></a>

### D05 — Saffire-I
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-98

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Puține arderi; calibrări și metrologie, nu un corpus mare de incendii independente.


<a id="source-d06"></a>

### D06 — Saffire-II
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-99

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Nouă înregistrări de probe în auditul anterior; materiale eterogene.


<a id="source-d07"></a>

### D07 — Saffire-III
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-100

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Puține arderi; identități și unități de reconciliat.


<a id="source-d08"></a>

### D08 — FLEX-2
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-68

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Arhivă mare, alt pipeline; opt intrări în sumar descriu familii, nu numărul încercărilor.


<a id="source-d09"></a>

### D09 — SPICE
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-107

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 469 rânduri, 34 duplicate exacte, 151 perechi dată/aprindere în auditul anterior.


<a id="source-d10"></a>

### D10 — Cool-flame dynamics / CFD analysis
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-117

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Curbe publicate experimentale și simulate; posibile suprapuneri cu FLEX.


<a id="source-d11"></a>

### D11 — Computational/experimental SPICE analysis
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-60

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Auditul anterior a găsit numai un mic ZIP de metadate, nu câmpuri CFD.


<a id="source-d12"></a>

### D12 — Concurrent spread / FGM / tomography
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-62

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Auditul anterior a găsit numai metadate, nu un corpus de câmpuri.


<a id="source-d13"></a>

### D13 — ACME Flame Design
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-10

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 207+120 intrări în tabelele anterioare; Tad calculat, nu temperatură măsurată; licență neclarificată.


<a id="source-d14"></a>

### D14 — ACME Burning Rate Emulator
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-20

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 18 rezumate zilnice, 194 aprinderi însumate; registry pe run neauditat.


<a id="source-d15"></a>

### D15 — ACME S-Flames
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-23

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** 18 rezumate zilnice, 276 aprinderi însumate; nu sunt 276 exemple deja calificate.


<a id="source-d16"></a>

### D16 — Cool-flame transitions / counterflow ground reference
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-142

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Geometrie la 1 g, separată de ISS.


<a id="source-d17"></a>

### D17 — Cool Flames Investigation
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-39

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Sursă suplimentară din catalogul concurentului; nu este cohortă Ignis calificată.


<a id="source-d18"></a>

### D18 — SAME-R / smoke detection
Sursă: https://psi.nasa.gov/physci/repo/data/investigations/PSI-101

**Statut:** Inventar anterior / candidat; nu nouă descărcare în această consolidare.
**Rol:** Semnale de detector; nu transformați o tensiune într-o etichetă de alarmă inventată.


<a id="source-d19"></a>

### D19 — SoFIE-GEL
Sursă: https://science.nasa.gov/biological-physical/investigations/sofie-gel/

**Statut:** Descriere publică moștenită; corpusul complet de măsurători nu a fost localizat.
**Rol:** 54 arderi pe 17 sfere reutilizate în documentația citată; nu promisiune de acces la date.


<a id="source-d20"></a>

### D20 — ZARM / FLARE — arhivă termică
Sursă: https://zenodo.org/records/18479713

**Statut:** Manifest/index moștenit, nu toate fișierele MAT inspectate.
**Rol:** 30 active MAT, 31 intrări index, CC BY-NC 4.0 raportate anterior; domeniu separat.


<a id="source-p01"></a>

### P01 — Dietrich et al. — Droplet Combustion Experiments Aboard the International Space Station
Sursă: https://doi.org/10.1007/s12217-014-9372-2

**Statut:** Referință fizică folosită în V1.
**Rol:** Context FLEX și limitele aproximării D².


<a id="source-p02"></a>

### P02 — Dynamics of Water Absorption and Evaporation During Methanol Droplet Combustion in Microgravity
Sursă: https://ntrs.nasa.gov/citations/20130010979

**Statut:** Referință NASA moștenită.
**Rol:** Diametrul net nu este masa de metanol pur rămasă.


<a id="source-p03"></a>

### P03 — NASA — studiu despre ardere/cool flames după stingerea flăcării vizibile
Sursă: https://ntrs.nasa.gov/citations/20140006750

**Statut:** Referință NASA moștenită.
**Rol:** Sfârșit vizibil versus încetarea tuturor reacțiilor.


<a id="source-p04"></a>

### P04 — Olson și Ferkul — Microgravity Flammability of PMMA Rods in Concurrent Flow
Sursă: https://ntrs.nasa.gov/citations/20150008961

**Statut:** Referință NASA și audit de protocol anterior.
**Rol:** Ventilație, dwell, quenching/blowoff; nu transfer direct la film planar.


<a id="source-p05"></a>

### P05 — Link et al. — The Effect of Gravity on Flame Spread over PMMA Cylinders
Sursă: https://www.nature.com/articles/s41598-017-18398-4

**Statut:** Studiu 2018 și supliment analizate anterior; conflict de unități nerezolvat.
**Rol:** 52 segmente, mai puține probe; validare scalară numai după reconciliere.


<a id="source-p06"></a>

### P06 — NASA BASS-II summary report
Sursă: https://ntrs.nasa.gov/citations/20210011385

**Statut:** Referință de protocol moștenită.
**Rol:** Context și identificarea subseturilor BASS.


<a id="source-p07"></a>

### P07 — NASA BASS sensor analysis
Sursă: https://ntrs.nasa.gov/citations/20150019858

**Statut:** Referință de protocol moștenită.
**Rol:** Măsurare și controluri.


<a id="source-p08"></a>

### P08 — NASA-STD-6001 — materials test standard record
Sursă: https://standards.nasa.gov/node/255

**Statut:** Versiunea aplicabilă trebuie verificată.
**Rol:** Ignis nu certifică materiale și nu înlocuiește încercările standard.


<a id="source-p09"></a>

### P09 — NASA — Flammability of Materials on the Moon
Sursă: https://www.nasa.gov/glenn/glenn-expertise-space-exploration/physical-sciences-program/combustion-science/flammability-of-materials-on-the-moon-fm2/

**Statut:** Context de misiune; distingeți planificat de observat.
**Rol:** Nu extrapolați automat predicțiile ISS la Lună.


<a id="source-l01"></a>

### L01 — Fourier Neural Operator for Parametric Partial Differential Equations
Sursă: https://arxiv.org/abs/2010.08895

**Statut:** Metodă fundamentală, referință cunoscută; nu benchmark Ignis.
**Rol:** Operatori funcție→funcție; inspirație pentru BASS.


<a id="source-l02"></a>

### L02 — Factorized Fourier Neural Operators
Sursă: https://arxiv.org/abs/2111.13802

**Statut:** Referință din review-ul anterior.
**Rol:** Factorizare spectrală; nu necesară pentru un front 1D mic.


<a id="source-l03"></a>

### L03 — DeepONet — Learning nonlinear operators via DeepONet based on the universal approximation theorem of operators
Sursă: https://doi.org/10.1038/s42256-021-00302-5

**Statut:** Referință fundamentală.
**Rol:** Branch/trunk pentru operatori; alternativă evaluată conceptual.


<a id="source-l04"></a>

### L04 — Fourier Neural Operator with Learned Deformations for PDEs on General Geometries / Geo-FNO
Sursă: https://arxiv.org/abs/2207.05209

**Statut:** Referință din review-ul anterior.
**Rol:** Geometrii variabile; cost suplimentar fără țintă corespunzătoare în nucleul actual.


<a id="source-l05"></a>

### L05 — Learning Data-Efficient and Generalizable Neural Operators via Fundamental Physics Knowledge
Sursă: https://arxiv.org/abs/2602.15184

**Statut:** 2026: titlu/statut și review de metode moștenite; revalidarea identității este în registrul de acces.
**Rol:** Pretraining pe fizică simplificată; exemplul smoke/ScalarFlow nu validează combustia NASA.


<a id="source-l06"></a>

### L06 — Learning Neural Operators from Partial Observations via Latent Autoregressive Modeling (LANO)
Sursă: https://arxiv.org/abs/2601.15547

**Statut:** 2026: review anterior; nu checkpoint executat sau transfer NASA verificat.
**Rol:** Observații parțiale și latent autoregresiv; inspirație, nu dependență.


<a id="source-l07"></a>

### L07 — Neural Operator Processes for Probabilistic Operator Learning under Partial Observations
Sursă: https://arxiv.org/abs/2606.22946

**Statut:** 2026: referință și analiză anterioară; acces/implementare de reconfirmat.
**Rol:** Context condiționat probabilistic; incertitudinea necesită validare independentă.


<a id="source-l08"></a>

### L08 — Koopman-inspired operator learning for intrinsic flame instabilities
Sursă: https://doi.org/10.1016/j.compfluid.2026.107165

**Statut:** 2026: sursă publisher/metode moștenite; cod/ponderi neexecutate.
**Rol:** Flame-front premixed instability diferit de picături și arderea solidelor.


<a id="source-l09"></a>

### L09 — RFNO-PINN: A Physics-Informed Neural Operator for Predicting Multi-Species Combustion Dynamics
Sursă: https://www.sciencedirect.com/science/article/pii/S1540748926001501

**Statut:** Pistă 2026 la nivel publisher; reproducibilitate completă neconfirmată.
**Rol:** Cere stări de specii/temperatură absente din observabilele centrale.


<a id="source-l10"></a>

### L10 — Test-time Generalization for Physics through Neural Operator Splitting
Sursă: https://proceedings.mlr.press/v306/serrano26a.html

**Statut:** Pistă furnizată; identitate și metode neconfirmate independent în consolidare.
**Rol:** Nu controlează arhitectura și nu susține afirmații de performanță.


<a id="source-l11"></a>

### L11 — Principled approaches for extending neural architectures to function spaces for operator learning
Sursă: https://www.nature.com/articles/s42256-026-01267-z

**Statut:** Pistă furnizată; detalii bibliografice/metode de verificat.
**Rol:** Nu dependență obligatorie.


<a id="source-l12"></a>

### L12 — Towards a Foundation Model for Partial Differential Equations Across Physics Domains (PDE-FM)
Sursă: https://arxiv.org/abs/2511.21861

**Statut:** Review anterior: atelier AI2ASE, nu automat conferința principală AAAI.
**Rol:** Pondere/cod și adaptare la câmpuri NASA neconfirmate.


<a id="source-l13"></a>

### L13 — Resolution-invariant temperature and species field reconstruction using factorized Fourier neural operator for turbulent jet flames
Sursă: https://doi.org/10.1016/j.combustflame.2025.114591

**Statut:** Publisher și sursă de autor identificate anterior; acces metodologic parțial.
**Rol:** Câmp OH informativ nu este RGB sau OH* necalibrat.


<a id="source-l14"></a>

### L14 — Physics-guided neural operator surrogate model of biomass pyrolysis gas combustion across variable compositions for relative flame-propagation screening in forestry deflagration
Sursă: https://doi.org/10.1016/j.psep.2026.109590

**Statut:** Pistă publisher 2026; metode/cod/ponderi complete neconfirmate.
**Rol:** Gaz de piroliză nu implică interfață solid–gaz învățată.


<a id="source-l15"></a>

### L15 — Uncovering flame physics with machine learning: application to the reaction rate in hydrogen flames
Sursă: https://arxiv.org/abs/2609.32552

**Statut:** Pistă/review anterior; artefacte și transfer de reconfirmat.
**Rol:** Suficiența observabilelor, nu permisiune de a inventa viteze de reacție.


<a id="source-l16"></a>

### L16 — Digital measurement of droplet flame diameter in microgravity combustion images using Segment Anything Model 2 with automatic prompt selection
Sursă: https://arxiv.org/abs/2607.16587

**Statut:** Review anterior la nivel rezumat; calibrări, independență și cod neconfirmate complet.
**Rol:** Segmentation + fitting geometric; nu temperatură sau chimie.


<a id="source-l17"></a>

### L17 — SAM 2: Segment Anything in Images and Videos
Sursă: https://arxiv.org/abs/2408.00714

**Statut:** Model pretrained și implementare publică identificate; nu rulate pentru validare NASA.
**Rol:** Backbone metrologic selectat, cu causalitate și calibrare.


<a id="source-l18"></a>

### L18 — Gaussians on Fire: High-Frequency Reconstruction of Flames
Sursă: https://arxiv.org/abs/2511.22459

**Statut:** Review anterior; licențe mixte ale codului/dependențelor de verificat.
**Rol:** Reconstrucție de aspect multiview, nu forecast fizic sau termometrie.


<a id="source-l19"></a>

### L19 — Loss-Driven Bayesian Active Learning
Sursă: https://arxiv.org/abs/2604.11995

**Statut:** Pistă 2026 și proceedings furnizate; niveluri de acces diferite în V1/V2.
**Rol:** Acquisition legată de loss; derivăm independent utilitatea folosită.


<a id="source-l20"></a>

### L20 — Neural Controlled Differential Equations for Irregular Time Series
Sursă: https://arxiv.org/abs/2005.08926

**Statut:** Metodă fundamentală.
**Rol:** Alternativă encoder pentru observații neregulate, nu necesară în primul release.


<a id="source-l21"></a>

### L21 — Universal Differential Equations for Scientific Machine Learning
Sursă: https://arxiv.org/abs/2001.04385

**Statut:** Metodă fundamentală.
**Rol:** Mecanism + rezidual învățat; inspirația FLEX.


<a id="source-l22"></a>

### L22 — Neural Spline Flows
Sursă: https://arxiv.org/abs/1906.04032

**Statut:** Metodă fundamentală.
**Rol:** Densități condiționate evaluabile și Jacobian exact; accelerare opțională.


<a id="source-l23"></a>

### L23 — Automatic Posterior Transformation for Likelihood-Free Inference
Sursă: https://arxiv.org/abs/1905.07488

**Statut:** Metodă fundamentală.
**Rol:** Training de densitate condiționată; corecția runtime este definită explicit.


<a id="source-l24"></a>

### L24 — Flow Matching for Generative Modeling
Sursă: https://arxiv.org/abs/2210.02747

**Statut:** Metodă fundamentală.
**Rol:** Alternativă generativă, respinsă pentru costul suplimentar în 2–12 dimensiuni.


<a id="source-l25"></a>

### L25 — Distribution-free inference with hierarchical data
Sursă: https://arxiv.org/abs/2306.06342

**Statut:** Review anterior de metodă/cod.
**Rol:** Grupuri independente și exchangeability; cadrele nu înlocuiesc grupurile.


<a id="source-l26"></a>

### L26 — Weak SINDy for Partial Differential Equations
Sursă: https://pmc.ncbi.nlm.nih.gov/articles/PMC8570254/

**Statut:** Metodă și implementare identificate anterior.
**Rol:** Diagnostic al dinamicilor, fără afirmația identificării unice a mecanismului.


<a id="source-l27"></a>

### L27 — Guiding diffusion models to reconstruct flow fields from sparse data
Sursă: https://arxiv.org/abs/2510.19971

**Statut:** Review anterior; licența/implementarea exactă de reconfirmat.
**Rol:** Reconstrucție de câmpuri cu observații fizice, nu RGB→termochemie.


<a id="source-l28"></a>

### L28 — Walrus — pretrained continuum dynamics
Sursă: https://arxiv.org/abs/2511.15684

**Statut:** Metadate și pistă de lucrare moștenite; nu ponderi executate.
**Rol:** Foundation model pentru câmpuri compatibile, nu nucleu obligatoriu.


<a id="source-l29"></a>

### L29 — Poseidon: Efficient Foundation Models for PDEs
Sursă: https://arxiv.org/abs/2405.19101

**Statut:** Cod/ponderi identificate anterior; rights per artefact de reconfirmat.
**Rol:** Intrări de câmp; ponderi NC raportate anterior, nu dependență obligatorie.


<a id="source-l30"></a>

### L30 — PDE-JEPA
Sursă: https://arxiv.org/abs/2609.34715

**Statut:** Pistă furnizată, acces anterior la nivel abstract.
**Rol:** Nu se presupune că există checkpoint NASA reutilizabil.


<a id="source-l31"></a>

### L31 — Phys-JEPA
Sursă: https://arxiv.org/abs/2606.16076

**Statut:** Pistă furnizată, acces anterior de descoperire.
**Rol:** Exploratoriu, fără rol obligatoriu.


<a id="source-l32"></a>

### L32 — ChemKANs for combustion chemistry modeling and acceleration
Sursă: https://doi.org/10.1039/D5CP02009C

**Statut:** Pistă publisher 2025.
**Rol:** Kinetică chimică cu stări corespunzătoare; nu înlocuiește diametrul măsurat.


<a id="source-l33"></a>

### L33 — Extended Fourier Neural Operators to learn stiff chemical kinetics under unseen conditions
Sursă: https://doi.org/10.1016/j.combustflame.2024.113847

**Statut:** Pistă publisher, articol 2025 în review-ul anterior.
**Rol:** Stiff chemistry, nu aceeași problemă ca inferența optică.


<a id="source-l34"></a>

### L34 — Joint models / dynamic prediction from longitudinal and event-time data
Sursă: https://arxiv.org/abs/1303.2797

**Statut:** Referință metodologică din V1.
**Rol:** Precedent pentru legarea traiectoriei de evenimente; Ignis nu inventează joint modeling.


<a id="source-l35"></a>

### L35 — A Gentle Introduction to Conformal Prediction and Distribution-Free Uncertainty Quantification
Sursă: https://arxiv.org/abs/2107.07511

**Statut:** Referință metodologică.
**Rol:** Acoperire cu ipoteze; nu garanție în afara domeniului.


<a id="source-l36"></a>

### L36 — Student-t Processes as Alternatives to Gaussian Processes
Sursă: https://arxiv.org/abs/1402.4306

**Statut:** Referință metodologică pentru distribuția t multivariată.
**Rol:** Likelihood coerent pe timpi neregulați; amestec de scară per run.


<a id="source-l37"></a>

### L37 — The No-U-Turn Sampler: Adaptively Setting Path Lengths in Hamiltonian Monte Carlo
Sursă: https://arxiv.org/abs/1111.4246

**Statut:** Referință metodologică pentru NUTS.
**Rol:** Costul depinde de geometria posteriorului, nu doar de numărul ponderilor.


<a id="source-l38"></a>

### L38 — A Bayesian approach to estimate flame spread model parameters over cylindrical PMMA samples under various gravity conditions
Sursă: https://doi.org/10.1016/j.combustflame.2024.113828

**Statut:** Pistă publisher 2025; metode complete/cod neconfirmate.
**Rol:** Precedent pentru calibrare fizică; nu transfer direct a posteriorului la ISS.


<a id="source-l39"></a>

### L39 — Opposed flame spread limit over PMMA rods in normal and micro gravity
Sursă: https://doi.org/10.1016/j.firesaf.2026.104719

**Statut:** Pistă fizică 2026 moștenită.
**Rol:** Definițiile pragurilor diferă; nu etichetă universală de siguranță.


<a id="source-l40"></a>

### L40 — Flame extinction of thermally thick PMMA in Earth gravity versus microgravity
Sursă: https://doi.org/10.1016/j.firesaf.2025.104625

**Statut:** Pistă publisher din review-ul anterior.
**Rol:** Thick PMMA, domeniu diferit de filmul planar inițial.


<a id="source-k01"></a>

### K01 — MicroFire Atlas — model source snapshot
Sursă: https://github.com/A-K-M-Asifuzzaman/MicroFire-Atlas/blob/acb9aaa69486154da657c61469d3a1886e4c12b1/apps/web/lib/model-lab.ts

**Statut:** Cod inspectat anterior, nu benchmark reprodus aici.
**Rol:** BASS classifier, grouped evaluation și abstention; grupare PMMA imperfect cunoscută.


<a id="source-k02"></a>

### K02 — MicroFire Atlas — Model Lab
Sursă: https://microfire-atlas.vercel.app/model-lab

**Statut:** Demo inspectat anterior; starea live de reverificat.
**Rol:** Punct de referință pentru explicații și limite.


<a id="source-k03"></a>

### K03 — Fotia — model serving
Sursă: https://github.com/AnteurAbderraouf/Fotia/blob/01e3842cdfd55be45bbd15ec1ab6e8d0816b3f8f/flame/dashboard/predict.py

**Statut:** Cod inspectat anterior.
**Rol:** Multimodel real: logistic, ridge, random forest. Nu simplu RAG.


<a id="source-k04"></a>

### K04 — Fotia — evaluation
Sursă: https://github.com/AnteurAbderraouf/Fotia/blob/01e3842cdfd55be45bbd15ec1ab6e8d0816b3f8f/flame/models/common.py

**Statut:** Cod inspectat anterior.
**Rol:** Stratified random folds în acest modul, nu afirmație despre tot repository-ul.


<a id="source-k05"></a>

### K05 — Fotia — ACME loader
Sursă: https://github.com/AnteurAbderraouf/Fotia/blob/01e3842cdfd55be45bbd15ec1ab6e8d0816b3f8f/flame/loaders/psi159.py

**Statut:** Cod inspectat anterior.
**Rol:** Separă corect inputuri de rezultate; nu criticăm eronat leakage neexistent.


<a id="source-k06"></a>

### K06 — Fotia — FLEX droplet dynamics baseline
Sursă: https://github.com/AnteurAbderraouf/Fotia/blob/01e3842cdfd55be45bbd15ec1ab6e8d0816b3f8f/flame/models/droplet_burn.py

**Statut:** Cod inspectat anterior.
**Rol:** Regresie K și lege D²; nu aceeași sarcină ca forecast din prefix.


<a id="source-k07"></a>

### K07 — FLARENEXUS
Sursă: https://github.com/KhalidHasan03/Flame-in-Freefall

**Statut:** README/cod inspectate anterior, nu rulare completă în consolidare.
**Rol:** Versiunea inspectată descria demonstrație sintetică.


<a id="source-k08"></a>

### K08 — ZeroG Ignis
Sursă: https://github.com/ShaharabiTuhin/ZeroG-Ignis

**Statut:** README/cod inspectate anterior.
**Rol:** Interfață/analist; predictor fizic propriu validat neidentificat.


<a id="source-k09"></a>

### K09 — Flame in Freefall — Mehedi752
Sursă: https://github.com/Mehedi752/Flame-in-Freefall

**Statut:** README/cod inspectate anterior.
**Rol:** 31 records/~25 publicații și analist pe reguli în versiunea inspectată.


<a id="source-t01"></a>

### T01 — JAX — compilation
Sursă: https://docs.jax.dev/en/latest/jit-compilation.html

**Statut:** Documentație oficială, versiune fixată la build.
**Rol:** JIT al calculelor numerice.


<a id="source-t02"></a>

### T02 — JAX — benchmarking
Sursă: https://docs.jax.dev/en/latest/benchmarking.html

**Statut:** Documentație oficială.
**Rol:** Sincronizare, separare compile/load/compute.


<a id="source-t03"></a>

### T03 — JAX — X64
Sursă: https://docs.jax.dev/en/latest/default_dtypes.html

**Statut:** Documentație oficială.
**Rol:** Precizie explicită, nu presupunerea implicită de float64.


<a id="source-t04"></a>

### T04 — JAX — GPU memory allocation
Sursă: https://docs.jax.dev/en/latest/gpu_memory_allocation.html

**Statut:** Documentație oficială.
**Rol:** Controlul prealocării când sunt mai multe framework-uri.


<a id="source-t05"></a>

### T05 — NumPyro MCMC
Sursă: https://num.pyro.ai/en/stable/mcmc.html

**Statut:** Documentație oficială.
**Rol:** Lanțuri, dispozitive, diagnostice și configurație.


<a id="source-t06"></a>

### T06 — Equinox
Sursă: https://docs.kidger.site/equinox/

**Statut:** Documentație oficială.
**Rol:** Rețele JAX mici.


<a id="source-t07"></a>

### T07 — Diffrax
Sursă: https://docs.kidger.site/diffrax/

**Statut:** Documentație oficială.
**Rol:** Integrare/diferențiere unde este efectiv necesară.


<a id="source-t08"></a>

### T08 — Tokio — spawn_blocking
Sursă: https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html

**Statut:** Documentație oficială.
**Rol:** Nu blocați firele async cu integrare/NUTS.


<a id="source-t09"></a>

### T09 — Rayon
Sursă: https://docs.rs/rayon/latest/rayon/

**Statut:** Documentație oficială.
**Rol:** Pool numeric limitat.


<a id="source-t10"></a>

### T10 — Axum
Sursă: https://docs.rs/axum/latest/axum/

**Statut:** Documentație oficială.
**Rol:** API Rust.


<a id="source-t11"></a>

### T11 — SQLite FTS5
Sursă: https://www.sqlite.org/fts5.html

**Statut:** Documentație oficială.
**Rol:** BM25 și convenția ordinii scorului.


<a id="source-t12"></a>

### T12 — SQLite WAL
Sursă: https://www.sqlite.org/wal.html

**Statut:** Documentație oficială.
**Rol:** Separare catalog read-only / analiză; nu WAL multi-host pe network volume.


<a id="source-t13"></a>

### T13 — SimSIMD
Sursă: https://github.com/ashvardanian/SimSIMD

**Statut:** Implementare identificată; pin la versiunea folosită.
**Rol:** Scoring vectorial exact; benchmark crossover ANN.


<a id="source-t14"></a>

### T14 — Qwen3-Embedding-0.6B
Sursă: https://huggingface.co/Qwen/Qwen3-Embedding-0.6B

**Statut:** Card/ponderi identificate anterior; pin hash/licență.
**Rol:** Embeddings pretrained, fără fine-tuning obligatoriu.


<a id="source-t15"></a>

### T15 — Qwen3-Reranker-0.6B
Sursă: https://huggingface.co/Qwen/Qwen3-Reranker-0.6B

**Statut:** Card/ponderi identificate anterior; pin template.
**Rol:** Reranking opțional bounded.


<a id="source-t16"></a>

### T16 — Text Embeddings Inference
Sursă: https://huggingface.co/docs/text-embeddings-inference/index

**Statut:** Documentație oficială, compatibilitate de testat.
**Rol:** Serving batch/opțional, nu GPU permanent în bugetul de 100 USD.


<a id="source-t17"></a>

### T17 — vLLM
Sursă: https://docs.vllm.ai/

**Statut:** Documentație oficială, compatibilitate de testat.
**Rol:** Serving opțional; nu simultan cu joburi GPU fără buget de memorie.


<a id="source-t18"></a>

### T18 — SAM2 official repository
Sursă: https://github.com/facebookresearch/sam2

**Statut:** Implementare pretrained identificată anterior.
**Rol:** Metrologie și adaptare numai dacă ajută.


<a id="source-t19"></a>

### T19 — SAM2.1 Hiera-small
Sursă: https://huggingface.co/facebook/sam2.1-hiera-small

**Statut:** Metadate raportate anterior: ~46,1M, Apache-2.0; pin la build.
**Rol:** Nu reprezintă validare pe NASA.


<a id="source-t20"></a>

### T20 — PyTorch compile
Sursă: https://docs.pytorch.org/tutorials/intermediate/torch_compile_tutorial.html

**Statut:** Documentație oficială.
**Rol:** Optimizare doar a traseelor suportate, măsurată separat de compile.


<a id="source-t21"></a>

### T21 — uPlot
Sursă: https://github.com/leeoniya/uPlot

**Statut:** Implementare publică.
**Rol:** Grafice temporale Canvas, nu benchmark Ignis.


<a id="source-t22"></a>

### T22 — Apache ECharts
Sursă: https://echarts.apache.org/handbook/en/

**Statut:** Documentație oficială.
**Rol:** Vizualizări complementare lazy-loaded.


<a id="source-t23"></a>

### T23 — Cantera
Sursă: https://cantera.org/stable/

**Statut:** Software științific public.
**Rol:** Nu include automat un caz microgravitațional NASA calibrat.


<a id="source-t24"></a>

### T24 — NIST FDS
Sursă: https://pages.nist.gov/fds-smv/

**Statut:** Software științific public.
**Rol:** Nu se prezintă validarea terestră drept validare microgravity.


<a id="source-t25"></a>

### T25 — OpenFOAM
Sursă: https://www.openfoam.com/

**Statut:** Software științific public.
**Rol:** Simulator posibil de cercetare, nu dependență a release-ului de 100 USD.


<a id="source-t26"></a>

### T26 — NeuralOperator
Sursă: https://github.com/neuraloperator/neuraloperator

**Statut:** Framework identificat anterior.
**Rol:** Alternativă implementare; micul operator BASS poate fi explicit.


<a id="source-t27"></a>

### T27 — NVIDIA PhysicsNeMo
Sursă: https://github.com/NVIDIA/physicsnemo

**Statut:** Framework identificat anterior.
**Rol:** Nu adăugăm infrastructură fără un caz necesar.


<a id="source-t28"></a>

### T28 — FlameTracker
Sursă: https://github.com/combustionTools/flameTracker

**Statut:** GPL-3.0 raportat anterior; verificați exact redistribuirea.
**Rol:** Comparator offline pentru măsurare; nu copiăm automat cod în licența proiectului.


<a id="source-t29"></a>

### T29 — Chronos-2
Sursă: https://huggingface.co/amazon/chronos-2

**Statut:** Artefact din review-ul anterior; nu executat în Ignis.
**Rol:** Comparator time-series opțional.


<a id="source-t30"></a>

### T30 — MOTION
Sursă: https://github.com/JeongsLee/MOTION

**Statut:** Pistă cu licență raportată MIT; metodă/performanță neconfirmate.
**Rol:** Nu dependență esențială.


<a id="source-t31"></a>

### T31 — SAM3
Sursă: https://huggingface.co/facebook/sam3

**Statut:** Pistă mai nouă, rights custom/gated raportate anterior.
**Rol:** Nu selectat doar pentru recență.


<a id="source-r01"></a>

### R01 — Runpod — pricing
Sursă: https://www.runpod.io/pricing

**Statut:** Tarife din revizia bugetară anterioară; checkout trebuie reconfirmat.
**Rol:** 3090 0,50 USD/h, 4090 0,89 USD/h folosite ca ipoteze/plafon ofertă, nu rezervare.


<a id="source-r02"></a>

### R02 — Runpod — GPU models
Sursă: https://www.runpod.io/gpu-models

**Statut:** Tarife Community minime citate anterior, fără stoc autentificat.
**Rol:** Economii opționale, nu premisa bugetului.


<a id="source-r03"></a>

### R03 — Runpod — Pod pricing and storage
Sursă: https://docs.runpod.io/pods/pricing

**Statut:** Documentație oficială; verificați prețul aplicabil.
**Rol:** Cost de pod activ versus activitate GPU; stop și terminate.


<a id="source-r04"></a>

### R04 — Runpod — Network volumes
Sursă: https://docs.runpod.io/storage/network-volumes

**Statut:** Documentație oficială; suport cloud/DC de confirmat.
**Rol:** Persistență, cost și constrângere de datacenter.


<a id="source-j01"></a>

### J01 — TypeSafe — Jev models
Sursă: https://docs.typesafe.ai/models

**Statut:** Versiuni/prețuri istorice citate anterior; actualitatea de reconfirmat.
**Rol:** Router opțional prin API, nu simulator.


<a id="source-j02"></a>

### J02 — TypeSafe — Intent routing
Sursă: https://docs.typesafe.ai/patterns/intent-routing

**Statut:** Documentație furnizor.
**Rol:** Alegerea semantică a unui tool, după care se aplică gate-uri deterministe.


<a id="source-j03"></a>

### J03 — TypeSafe — Jev jaggedness
Sursă: https://docs.typesafe.ai/model-jaggedness/jev-1.13

**Statut:** Documentație a unei versiuni istorice.
**Rol:** Aritmetica și restricțiile de domeniu rămân în cod.

---

<a id="chapter-28"></a>

# 28. Receipt de acces la metadate publice

Aceasta este o verificare automată limitată a accesului HTTP și a titlului returnat. Nu dovedește că metodele au fost citite integral, că statutul peer-review este corect sau că există cod/ponderi utilizabile. Un eșec de acces nu dovedește inexistența lucrării ori nepublicarea resurselor. Un titlu diferit cere verificare manuală și nu trebuie trecut sub tăcere. Tarifele și regulile nu sunt confirmate doar prin încărcarea paginii.

| ID | HTTP | Stare | Titlu returnat / eroare |
|---|---:|---|---|
| C01 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='www.spaceappschallenge.org', port=443): Max retries exceeded with url: /2026/challenges/flame-in-freefall/ (Caused by NameResolutionError("HTTPSConnection(host='www.spaceappschallenge.org', port=443): Failed to resolve 'www. |
| L05 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2602.15184 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L06 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2601.15547 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L07 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2606.22946 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L10 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='proceedings.mlr.press', port=443): Max retries exceeded with url: /v306/serrano26a.html (Caused by NameResolutionError("HTTPSConnection(host='proceedings.mlr.press', port=443): Failed to resolve 'proceedings.mlr.press' ([Err |
| L11 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='www.nature.com', port=443): Max retries exceeded with url: /articles/s42256-026-01267-z (Caused by NameResolutionError("HTTPSConnection(host='www.nature.com', port=443): Failed to resolve 'www.nature.com' ([Errno -3] Tempora |
| L16 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2607.16587 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L19 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2604.11995 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L30 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2609.34715 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| L31 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='arxiv.org', port=443): Max retries exceeded with url: /abs/2606.16076 (Caused by NameResolutionError("HTTPSConnection(host='arxiv.org', port=443): Failed to resolve 'arxiv.org' ([Errno -3] Temporary failure in name resolutio |
| R01 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='www.runpod.io', port=443): Max retries exceeded with url: /pricing (Caused by NameResolutionError("HTTPSConnection(host='www.runpod.io', port=443): Failed to resolve 'www.runpod.io' ([Errno -3] Temporary failure in name reso |
| J01 | — | ACCESS_NOT_CONFIRMED | ConnectionError: HTTPSConnectionPool(host='docs.typesafe.ai', port=443): Max retries exceeded with url: /models (Caused by NameResolutionError("HTTPSConnection(host='docs.typesafe.ai', port=443): Failed to resolve 'docs.typesafe.ai' ([Errno -3] Temporary failure in n |

Receipt-ul JSON păstrează URL-ul final și metadatele disponibile. Marcajul de dată al runtime-ului din receipt nu schimbă cutoff-ul bibliografic al proiectului. Pentru o identitate conflictuală, documentul păstrează pista istorică, dar acea referință nu controlează nicio afirmație de performanță Ignis.
