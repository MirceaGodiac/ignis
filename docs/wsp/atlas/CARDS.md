# The 19 flight combustion investigations in NASA's PSI repository

One card per investigation, built by `tools/wsp/atlas/build_cards.py`. Objective and approach are NASA's own words from the
PSI metadata record. Counts and ranges come from the test catalogue and the file catalogue of this folder (conditions, names
and sizes; no outcome of a test was read). The fire-safety need of each investigation is a typed judgement, given with the
sentence of NASA's it rests on. Findings from NASA's documents are not in the cards yet.

## FLEX (PSI-69)

**Flame Extinguishment Experiment**. Combustion Integrated Rack (CIR)-Fluids and Combustion Facility (FCF), Multi-user Droplet Combustion Appartatus (MDCA). Family: droplets in the MDCA. DOI 10.60555/mbq8-0451. Licence CC0-1.0.

**Objective (NASA).** The FLEX (Flame Extinguishment Experiment) investigation aims to map the flammability limits of liquid fuels in microgravity environments, measure the effectiveness of various gaseous fire suppressants such as CO2, He, and SF6, and develop reliable predictive models for combustion limits under reduced gravity conditions. The mission seeks to validate and improve simplified models by incorporating detailed chemical reactions, heat transfer, and material movement, using both space-based data and extensive ground-based tests. Key flight experiment goals include obtaining high-resolution measurements of droplet burning rates, flame extinction behavior, heat radiation, soot levels, and temperature profiles, while systematically analyzing the effects of different ambient conditions and suppressant concentrations. These investigations are essential for advancing the fundamental understanding of combustion processes in microgravity and enhancing fire safety protocols for future space missions.

**Approach (NASA).** The FLEX experiments involve igniting and observing the burning behavior of relatively large droplets. A primary objective of the experiments is to determine the Limiting Oxygen Index (LOI) as a function of ambient pressure and determine how the presence of a suppressant influences the LOI . The basic outline of the procedure to construct this diagram is as follows: 1. Select the test pressure (0.5 − 1.5 atm) for the LOI tests. 2. Select a smaller droplet size (≈ 3 mm) 3. Select an ambient oxygen mole fraction that, based on ground-based testing, will support droplets burning to completion (as opposed to extinguishing at a finite droplet size). 4. Perform a droplet combustion test. 5. Increase the droplet size or repeat the test (if finite extinction diameter at flame extinction observed). 6. Continue steps 4 and 5 until flame exists for only a short time after ignition (radiative extinction). 7. Continue steps 4 thru 6 at a decreased ambient oxygen mole fraction. 8. Continue step 7 to an ambient oxygen mole fraction where the droplet does not ignite. There were ambient conditions where the extinction droplet sizes at all ambient oxygen mole fractions were too small to either be observed or measured with the available diagnostics. In this case, the determination of the LOI is similar, but more iterative. Instead of observing flame extinction, however, the procedure involved an iterative sequence of tests (burn to completion followed by a no ignition test) with progressively smaller increments in ambient oxygen mole fraction to determine the LOI . For the slow flow tests (in the flight experiment), the configuration of the flight hardware did not allow for adequate diagnostics over the entire test for a droplet in a uniform, steady convective flow. In these tests, the procedure will be to duplicate tests above which yield finite-sized extinction droplet diameters. Near the end of the tests, prior to flame extinction, the droplet will be subjected to a small, sub-buoyant, convective flow (imparted by translating the droplet at a slow, uniform speed). The goal of the experiment is then to see how the presence of this flow influences the extinction process. Note, a large number of the tests involve free droplets (as opposed to fiber-supported droplets) and a number of these tests will have droplets with a slow drift velocity. Data gathered from these tests to examine the influences of a small, sub-buoyant flow on both the burning and extinction process.

**Impacts, on Earth and in space (NASA).** The research impacts and Earth benefits of the FLEX mission are significant and multifaceted. By enhancing our understanding of microgravity combustion, the mission directly contributes to improved fire safety protocols for future space missions, ensuring the safety of crew members in environments where traditional fire suppression methods may be ineffective. On Earth, the insights gained from these experiments have the potential to advance fire safety in confined and reduced-gravity environments, such as submarines or high-altitude aircraft. The mission's findings on the effectiveness of various fire suppressants, including CO2, He, and SF6, in microgravity can lead to the development of more efficient and less harmful fire suppression systems, reducing the risk of toxic byproducts during fire events. Additionally, the study of ''cool flames'' and low-temperature combustion processes observed in microgravity can inform the design of advanced combustion engines, leading to improved fuel efficiency and reduced emissions in automotive and industrial applications. Overall, the FLEX mission's contributions to combustion science have the potential to enhance both space exploration safety and terrestrial fire safety technologies.

| | |
|---|---|
| Fuels or materials (rows) | methanol (92), heptane (64) |
| Phase and configuration | liquid_droplet; droplet_free |
| Sizes | initial droplet diameter 1.13 to 4.42 mm |
| Ambient oxygen | 0.12 to 0.34 (156 rows) |
| Pressure | 0 to 309 kPa (156 rows) |
| Imposed flow | none recorded |
| What varied between tests | carbon dioxide, helium, nitrogen, pressure, temperature, x o2, name, initial droplet diameter |
| Catalogue rows | 156 tests (the project's cohort: free-floating methanol and n-heptane droplets, train and validation runs; the investigation has more tests and fuels) |
| Files at NASA | 6,453 files, 921.4 GB; test ids in file names: 415 |
| Measurements per test | sensor series 278, images 411, video 120, processed data 269 |
| Crew atmospheres | earth iss: 35 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | suppression (named in the challenge brief); prevention (stated in NASA's objective for the investigation) |
| Towards partial gravity | nothing |
| Hand-written models | scored models: flex_methanol.wsp2 D3 and flex_heptane.wsp2 D2: diameter forecasts scored on validation days (CRPS 40.0 and 46.1 µm; development numbers) |
| Closest investigations | FLEX-2 (same hardware family, same fuel or material: heptane, same need: prevention, suppression); CFI (same hardware family, same need: prevention, suppression) |

**How the tests differ.** FLEX burned single n-heptane or methanol droplets, planned at about 3 and 5 mm initial diameter. Between tests NASA varied ambient pressure (planned 0.5 to 1.5 atm, plus a 2 to 3 atm radiative-environment set), lowered the oxygen mole fraction step by step toward the limiting oxygen index, and added CO2 or helium as suppressant; SF6 was planned, but the flight table lists only CO2 and helium. Droplets were free or fiber-supported, quiescent or slowly translated (planned 1.5 cm/s).

**Instruments.** MDCA radiometer package (broad band 0.6-5.0 um; narrow band H2O at 1.87 um) (Flame radiation); Sony colour camera package with turning mirror and zoom lens (Colour video of droplet burning and flame); HiBMS camera views (Backlit droplet view and soot temperature); High Bit-Depth Multispectral Package with tunable filter (Soot temperature and soot volume fraction); CIR Illumination Package (fiber-coupled laser diode, 650-660 nm, 90 mm collimated beam) (Backlight for droplet and soot imaging); Low Light Level UV camera (intensified, sensitive to OH at 310 nm) (OH* chemiluminescence images of the flame); Chamber pressure transducers (PT16, PT17, PT28, PT29) and temperature channels (TM1-TM4) (Chamber pressure (psia) and temperature (deg C)); SAMS acceleration sensor (CIR tri-axial sensor head on the optics bench) (Acceleration environment).

**Findings, from NASA's documents.**

- In ISS tests, n-heptane droplets burned with a hot flame, lost the visible flame by radiative extinction, then kept reacting in a cool-flame stage without visible flame that ended in diffusive extinction; a detailed-kinetics model reproduces this. *(NASA states; PSI-69_Presentations_1G17-070HE-0166.pdf, p. 1)*
- Simulations in the same study indicate that heat loss initiates the transition to cool-flame burning, and that adding CO2 or helium to the ambient triggers it through enhanced heat loss. *(NASA states; PSI-69_Presentations_1G17-070HE-0166.pdf, p. 6)*
- For n-octane and n-decane droplets in air (ISS and drop tower), burning rate and flame standoff ratio fall and soot standoff ratio rises with initial diameter; radiative loss explains the large-droplet trend, and the support fiber raises burning rate for 1-2 mm droplets. *(NASA states; PSI-69_Presentations_2G11-070HE-0310.pdf, p. 10)*
- In methanol droplet flames, combustion water condenses into the droplet and later evaporates; the theory presented links this water to a falling flame temperature until the reaction can no longer be sustained. *(NASA states; PSI-69_Presentations_20130010979.pdf, p. 4)*
- The requirements document argues that radiative and conductive losses near extinction give a minimum limiting oxygen index at a critical droplet diameter, and that this droplet LOI is a conservative fire-safety limit. *(NASA states; PSI-69_Engineering Documents_FLEX_EIRD_v4.pdf, p. 16)*
- Spacecraft practice stops ventilation on a fire alarm, but the requirements document notes that crew movement and extinguisher discharge induce flows of several to tens of cm/s, leaving open how such flow changes the extinguishing concentration. *(NASA states; PSI-69_Engineering Documents_FLEX_EIRD_v4.pdf, p. 8)*

**Why it matters for fire safety** (NASA states). NASA states FLEX maps the flammability limits of liquid fuels in microgravity and measures the effectiveness of gaseous suppressants (CO2, He, SF6) to improve fire-safety protocols for space missions; its requirements document links this to the ISS suppressant trade studies that chose CO2 extinguishers and to a flooding goal of bringing oxygen below the limiting oxygen index.

- *suppression, named in the challenge brief:* "Suppression was studied in the Flame Extinguishment Experiment (FLEX) through dilution of the chamber atmosphere with carbon dioxide and helium, both of which suppress fire."
- *prevention, stated in NASA's objective for the investigation:* "The FLEX (Flame Extinguishment Experiment) investigation aims to map the flammability limits of liquid fuels in microgravity environments"

## FLEX-2 (PSI-68)

**Flame Extinguishment Experiment - 2**. Combustion Integrated Rack (CIR), Multi-user Droplet Combustion Appartatus (MDCA). Family: droplets in the MDCA. DOI 10.60555/byp8-h018. Licence CC0-1.0.

**Objective (NASA).** The primary goal of the FLEX-2 flight experiments is to extend the results of FLEX-1 to fuels and environmental conditions that mimic real combustor conditions, as well as, provide benchmark experimental data on bi-component droplet combustion over a wide range of initial conditions in order to validate numerical simulations and to develop new theoretical models. Detailed experimental objectives can be further itemized as: 1. Investigate the role of liquid-phase mixing on the burning characteristics of bi-component fuel droplets. 2. Examine the steady and unsteady liquid and gas-phase phenomena in pure fuel combustion. 3. Study flame extinction mechanisms and their boundaries. 4. Analyze the soot formation process, including soot volume fraction soot-shell dynamics. 5. Determine the influence of droplet interactions on the flammability limits and extinction behavior of droplet arrays. 6. Validate theoretical models of droplet combustion through comprehensive experimental data collection in microgravity environments.

**Approach (NASA).** The chamber was filled with the appropriate atmosphere, which was varied in pressure between 0.5 atm to 3.0 atm and O2 concentration between 0.1 to 0.4 mole fraction with the diluent gas making up the balance. A settling time of approximately 2 minutes elapsed prior to initiating the test in order to ensure that the temperature and pressure of the chamber gases have stabilized. This settling time was followed by the dispensing of a predetermined amount of fuel (based on the target droplet size) onto the support fiber for those tests in which the droplet is not freely deployed. When sufficient fuel has been dispensed the dispensing needles were retracted and a dwell period of at least 10 seconds was allowed for the droplet internal fluid motion induced by deployment to subside. This was then followed by initiating power to the ignitor for a selectable amount of time ranging from 1 second to 5 seconds after which the ignitor was retracted from the field of view. If the flow field is to be generated by translating the droplet then droplet motion would commence at the same time that the ignitor is retracted. A near real-time download of the color camera video was employed in order to verify successful droplet deployment, ignition, and overall progress of the experiment. Pressure and temperature data of the chamber environment was also recorded in near real time. The overall investigation approach can be summarized as: 1. Test parameters are determined, including pressure, temperature, fuel mixture, and humidity. 2. Fill the combustion chamber with the selected gas mixtures. 3. Deploy the droplet(s) carefully controlling the droplet diameter. 4. Allow the droplet to stabilize in the microgravity environment. 5. Ignite the droplet in a uniform and controlled fashion. 6. Monitor combustions and track flame evolution using high speed photography. 7. Monitor and adjust experimental conditions as needed. 8. Analyze data. The independently varied experimental parameters during FLEX-2 experiments were 1) Fuel type 2) Ambient gas composition 3) Ambient Pressure 4) Droplet initial diameter 5) Droplet translation velocity (and acceleration) 6) Droplet spacing and initial size in a binary linear array.

**Impacts, on Earth and in space (NASA).** The combustion of liquid fuels is the overwhelming energy source in the transportation sector. Insights gained from these studies can lead to more efficient and cleaner combustion technologies which are crucial for reducing pollution and greenhouse gas emissions. By improving fuel efficiency and developing better fuels, these experiments can contribute to: 1) More sustainable energy production. 2) Future combustor designs with minimal carbon footprint, maximum fuel efficiency and less pollutant emissions. 3) Allow quantitative evaluation of the performance of future fuels in combustors through the development of surrogates (mixtures of pure fuels that simulate the behavior of real fuels). 4) Validation of jet engine models by engine manufacturers with support from prior droplet results.

| | |
|---|---|
| Fuels or materials (rows) | propanol+glycerol (2), isooctane (1), n_octane (1), heptane (1), decane (1), ethanol (1), heptane+isooctane (1) |
| Phase and configuration | liquid_droplet; droplet_unspecified |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | none recorded |
| What varied between tests | pressure, x o2, speed, name, initial droplet diameter |
| Catalogue rows | 8 matrix rows (every row of NASA's experiment table) |
| Files at NASA | 10,077 files, 1,221.9 GB; test ids in file names: 596 |
| Measurements per test | sensor series 586, images 535, video 0, processed data 0 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | suppression (foundational); prevention (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | not usable yet: the droplet models take the fuel by name; n-heptane tests exist but their diameters are only in images |
| Closest investigations | FLEX (same hardware family, same fuel or material: heptane, same need: prevention, suppression); CFI (same hardware family, same need: prevention, suppression) |

**How the tests differ.** FLEX-2 varied six parameters independently: fuel type (pure fuels such as ethanol, decane, heptane, n-octane and iso-octane, plus bi-component and surrogate mixtures), ambient gas (oxygen in nitrogen or helium), pressure (0.5 to 3.0 atm), initial droplet diameter (planned 1.5 to 5 mm), droplet translation velocity for slow convection (planned 0.1 to 5 cm/s), and spacing and size in binary droplet arrays. NASA's approach gives oxygen 0.1 to 0.4 mole fraction; the experimental table lists 0.17 to 0.3.

**Instruments.** Broadband radiometer (Dexter S60M thermopile, KRS-5 window, argon fill) (Broadband flame thermal radiation); Narrowband radiometer (Dexter 2M thin-film thermopile, silicon long-wave-pass filter and sapphire window, about 6 micrometres) (Narrowband flame radiation); Video imaging at 30 fps (up to 100 fps for the droplet view) and radiometers sampled at 100 Hz (Droplet and flame diameter histories, extinction diameters, radiant emission); CIR diagnostics: high resolution/high frame rate camera, high bit depth multispectral camera with tunable filter, UV and IR low-light packages, laser-diode illumination package (Droplet, flame and soot imaging over the listed frame rates, fields of view and spectral ranges); FOMA Control Unit sensors (Chamber pressures, temperatures and gas flows); 3-column gas chromatograph (Chamber gas composition).

**Findings, from NASA's documents.**

- The SRD explains two extinction modes: radiative extinction of large flames by radiant loss and diffusive extinction of small flames by conductive loss; the boundary between them was known for heptane but not for ethanol or decane, which FLEX-2 set out to determine. *(NASA states; PSI-68_Science Documents_FLEX-2 SRD Final _ No Hypertext .pdf, p. 10)*
- In multicomponent droplets, capillary flows and differential vaporization produce flame contraction, which the Shaw and Williams theory describes through epsilon, a diffusion-time ratio. *(NASA states; PSI-68_Science Documents_FLEX-2 SRD Final _ No Hypertext .pdf, p. 11)*
- Soot affects droplet burning as a pyrolysis heat sink, a store of chemical energy and a radiation source, and sooting can cause radiative extinction; the SRD also notes that soot morphology matters for smoke detection in microgravity. *(NASA states; PSI-68_Science Documents_FLEX-2 SRD Final _ No Hypertext .pdf, p. 16)*
- Earlier droplet-array results reviewed in the SRD explain array extinction by the ratio of radiative loss to heat release in nitrogen and by residence time in helium; almost no theory exists for comparison. *(NASA states; PSI-68_Science Documents_FLEX-2 SRD Final.pdf, p. 23)*
- Earlier drop-tower, parabolic-flight and Spacelab experiments left no benchmark data for droplet burning in slow convective flow, the gap behind FLEX-2's slow-convection goal. *(NASA states; PSI-68_Science Documents_FLEX-2 SRD Final.pdf, p. 20)*
- Ground calibration of the FLEX-2 flight radiometers found the broadband fit within 5 percent at maximum irradiance but only within about 17 percent down to 0.0002 W/cm2, with larger scatter for the narrowband channel. *(NASA states; PSI-68_Calibration_Radiometer Calibration Test Report, Fluids.pdf, p. 43)*

**Why it matters for fire safety** (interpretation). NASA frames FLEX-2 as fundamental liquid-fuel combustion for cleaner, more efficient engines and surrogate fuels, returning from FLEX's fire-safety focus. Its fire-safety value is indirect: radiative and diffusive extinction boundaries for new fuels, flammability maps for interacting droplets, and soot data the SRD links to microgravity smoke detection.

- *suppression, foundational:* "Study flame extinction mechanisms and their boundaries."
- *prevention, foundational:* "Determine the influence of droplet interactions on the flammability limits and extinction behavior of droplet"

## CFI (PSI-39)

**Cool Flames Investigation**. Multi-User Droplet Combustion Apparatus (MDCA), Combustion Integrated Rack (CIR). Family: droplets in the MDCA. DOI 10.60555/m6pj-a914. Licence CC0-1.0.

**Objective (NASA).** The objectives of this investigation are to: 1) Examine the combustion behavior of n-dodecane and mixtures of n-dodecane/iso-dodecane at pressures ranging from 0.25 atm to 5 atm. 2) Analyze the effects of helium dilution on cool flame behavior. 3) Examine the behavior of combustion characteristics. 4) Investigate the relationship between droplet size, pressure, and flame extinction, and (5) understand the consequence of soot formation during cool flame combustion.

**Approach (NASA).** The Combustion Integrated Rack (CIR) contains the Multi-User Droplet Combustion Apparatus (MDCA) which facilitates the controlled combustion of fuel droplets. A system of gas bottles and fuel syringes delivers the desired gas mixture to the combustion chamber ranging from 0.25 to 0.5 atm. Droplets of fuel are dispensed and suspended between hypodermic needles or tethered using silicon-carbide fibers. A hot-wire igniter is positioned near the droplet for ignition. After ignition the wire is retracted. Fuels observed include n-dodecane, farnesane, and mixtures of n-dodecane/iso-dodecane. Fuels diluted with molecular oxygen, atomic nitrogen, and helium to study the influence on flame extinction and dynamics. Flames were observed with a black-and-white camera for droplet size measurements. A UV camera was used for capturing the OH* radical emissions and flame size. Other high resolution cameras were used for monitoring and documentation. Narrow-band and wide-band radiometers for analyzed flame radiation.

**Impacts, on Earth and in space (NASA).** A basic understanding of cool flames is important for understanding ignition phenomena of practical fuels and fuel additives. Cool flames play a key role in the development and selection of new fuels and the design of advanced engines such as clean, high-efficiency Homogeneous Charge Compression Ignition (HCCI) engines. The research will also help combustion chemists to develop new computational models to compute combustion performance in for many practical applications on earth, including an understanding of how cool flame reactivity determines octane and cetane number for fuels in spark ignition and diesel engines. Cool Flames Investigation (CFI) will provide critical validation for complex chemical kinetic mechanisms, advance the understanding of flame dynamics and instabilities in low-temperature combustion, and support the design of safer spacecraft systems by refining knowledge of combustion in microgravity environments.

| | |
|---|---|
| Fuels or materials (rows) | dodecane+iso_dodecane (103), dodecane (80), farnesane (44) |
| Phase and configuration | liquid_droplet; droplet_free, droplet_fiber_supported, droplet_unspecified |
| Sizes | initial droplet diameter 3.79 to 6.45 mm |
| Ambient oxygen | 0.17 to 0.21 (227 rows) |
| Pressure | 51 to 507 kPa (227 rows) |
| Imposed flow | none recorded |
| What varied between tests | helium, nitrogen, pressure, x o2, fiber, igniter power, igniter time, name |
| Catalogue rows | 227 tests (every row of NASA's experiment table) |
| Files at NASA | 2,812 files, 718.9 GB; test ids in file names: 133 |
| Measurements per test | sensor series 123, images 85, video 86, processed data 25 |
| Crew atmospheres | earth iss: 63 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | suppression (foundational); prevention (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | refused: n-dodecane, its mixtures and farnesane are outside the droplet models' fuels |
| Closest investigations | FLEX (same hardware family, same need: prevention, suppression); FLEX-2 (same hardware family, same need: prevention, suppression) |

**How the tests differ.** Single n-dodecane, n-dodecane/iso-dodecane or farnesane droplets, free-floating or tethered on a fiber, burned in the CIR chamber after a settling period, with no imposed flow. Across tests NASA varied fuel, pressure (0.25 to 5 atm in the final report; 0.5 to 5 atm in the data summary), oxygen mole fraction, the nitrogen diluent with helium partly replacing it, initial droplet size and support, and igniter power and duration.

**Instruments.** Black-and-white backlit camera (HiBMS) (droplet diameter history); UV camera (LLLUV, OH* at 310 nm) (OH* radical emission and hot-flame size); Narrow-band and wide-band radiometers (flame radiation); Colour and other high-resolution cameras (monitoring and documentation of the burn); Hot-flame and cool-flame camera settings (HIBMS, LLL, HIBMS-2) (per-test illumination and camera gains for imaging the bright hot flame and the faint cool flame); Chamber pressure and temperature sensors (chamber pressure and temperature, downlinked in near real time).

**Findings, from NASA's documents.**

- Cool flames burn quasi-steadily at about 700 to 900 K in the negative-temperature-coefficient (NTC) chemistry region, far cooler than hot flames above 1200 K. *(NASA states; PSI-39_Reports_CFI Science Final Report.pdf, p. 3)*
- FLEX results cited for CFI show that replacing nitrogen with helium raises droplet burning rates and shortens both hot and cool burning, and that fuel matters: n-alkanes show cool flames, methanol does not. *(NASA states; PSI-39_Science Documents_SRD_CFI.pdf, p. 26)*
- Applied to FLEX droplets, partial-burning theory predicts cool-flame burning rate and flame standoff ratio, and cool-flame extinction diameter correlates with oxygen concentration to the -3/2 power, but the approach fails at higher pressures. *(NASA states; PSI-39_Presentations_Aspects of Cool-Flame Supported Droplet Combustion in Microgravity_CFI.pdf, p. 17)*
- Cool-flame burning after hot-flame extinction produces partially oxidised fuel and may reignite, which the presenter calls a fire-safety concern; models mispredict the second (cool-flame) extinction. *(NASA states; PSI-39_Presentations_Cool Flames in Space, a Hot Prospect on Earth_CFI.pdf, p. 23)*
- Drop towers capture at most about 1 s of cool flame, so only long-duration microgravity such as the ISS gives quality cool-flame data over a wide range of conditions. *(NASA states; PSI-39_Science Documents_SRD_CFI.pdf, p. 34)*
- Earlier FLEX diagnostics were designed for hot flames, so no data on cool-flame structure existed; CFI was planned to add diagnostics aimed at the faint cool flame. *(NASA states; PSI-39_Science Documents_SRD_CFI.pdf, p. 29)*

**Why it matters for fire safety** (NASA states). NASA states that cool flames matter for spacecraft fire safety because a cool flame can persist after the hot flame goes out, generate combustible vapour and reignite, much like smouldering. So apparent extinction of a liquid-fuel fire in microgravity may not mean it is out.

- *suppression, foundational:* "Analyze the effects of helium dilution on cool flame behavior."
- *prevention, foundational:* "Investigate the relationship between droplet size, pressure, and flame extinction"

## ACME CFI-G (PSI-159)

**Advanced Combustion via Microgravity Experiments / Cool Flames Investigation with Gases**. ACME, Combustion Integrated Rack (CIR). Family: ACME burner flames in the CIR. DOI 10.60555/cyt1-kh64. Licence CC0-1.0.

**Objective (NASA).** 1. Plan and support flight testing on the International Space Station (ISS). 2. Determine the conditions for which cool diffusion flames occur for propane, n-butane, and n-pentane; normal and inverse flames; and a broad range of Zst, Tad, and fluent flow rate. 3. Support the flight tests with advanced CFD simulations. 4. Identify or develop a chemical kinetics mechanism that yields good agreement between the CFD predictions and the ISS measurements. 5. Disseminate the results as widely as possible.

**Approach (NASA).** Perform spherical diffusion flame tests aboard the International Space Station. Consider multiple fuels, a range of reactant dilutions, flow rates, and pressures for normal and inverse convection directions. Observe the flames with a broad suite of cameras, radiometers, photomultiplier tubes, and thermocouples.

**Impacts, on Earth and in space (NASA).** Combustion provides 86% of the world’s energy and an even higher fraction of the energy used for transportation. The internal combustion engines used for transportation generally have thermal efficiencies of 38% or lower because of thermal losses. These efficiencies may increase to 60% if future engines can take advantage of ultra-lean combustion. However, this progress is being impeded by the limited understanding of the chemistry of cool flames. Research into cool diffusion flames (whose peak temperatures are 500 – 1000 K) has grown exponentially since they were discovered in 2012. The measurements performed for this project will help to test and advance cool flame kinetics mechanisms. This should facilitate the development of more efficient internal combustion engines.

| | |
|---|---|
| Fuels or materials (rows) | butane (85), propane (64), ethane (57) |
| Phase and configuration | gas; unknown |
| Ambient oxygen | 0.18 to 0.40 (109 rows) |
| Oxygen in the injected stream | 0.15 to 0.85 (97 rows) |
| Pressure | 50 to 331 kPa (206 rows) |
| Imposed flow | none recorded |
| What varied between tests | pressure, x fuel ambient, x o2, tad, zst, name, flame orientation, fuel mass flow |
| Catalogue rows | 206 tests (the project's cohort: train and validation tests of the normal and inverse flames) |
| Files at NASA | 3,756 files, 2,085.6 GB; test ids in file names: 338 |
| Measurements per test | sensor series 337, images 331, video 335, processed data 0 |
| Crew atmospheres | earth iss: 14 inside; moon base rover: 17 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | scored models: acme_normal.wsp5 P1 and acme_inverse.wsp5 P1: when the flame ends and why, scored on validation days (extinction Brier 0.155 and 0.129; development numbers) |
| Closest investigations | ACME/Flame Design (same hardware family, same fuel or material: propane, same need: prevention); ACME s-Flame (same hardware family, same need: prevention); ACME BRE (same hardware family, same need: prevention) |

**How the tests differ.** Spherical diffusion flames burned on a 6.35 mm porous sphere in the constant-volume CIR chamber. Tests varied fuel (ethane, propane, n-butane; ethane replaced the planned n-pentane), flame type (normal, fuel from the burner; inverse, oxidizer from the burner), fuel and oxygen fractions with nitrogen dilution, pressure (about 0.5 to 3 atm) and burner flow rate. In all, 280 hot flames were ignited over 23 test days, and cool flames were sought after radiative extinction.

**Instruments.** ACME DataCam colour camera (BG7 and CH* filters) (flame imaging and colour pyrometry; BG7 filter thickness limited temperature accuracy); CFI intensified camera with formaldehyde filter (dim flame imaging, including cool flames, and flame diameters); HiBMs camera (red filter) (backlit imaging; in round two used to track soot on windows); CFI-G radiometer array (two fixed gains per sensor) (flame radiant heat flux, used to identify cool flames); Three photomultiplier tubes (230-700 nm, OH* and CH* bandpass, neutral density) (chemiluminescence of dim flames, with adjustable gain); Type-K thermocouple embedded in the porous spherical burner (burner temperature); Far-field thermocouple rake (types B and K) and two thermocouples at 200 mm (gas temperature around the spherical flame); Thin-filament pyrometry with SiC fibers (gas temperature profile through the flame); Mass flow controllers (fuel, oxidizer and nitrogen flow rates (poor at low flow)).

**Findings, from NASA's documents.**

- Spherical gas-fuelled cool diffusion flames were observed for the first time, but only for normal n-butane flames at about 2 bar and an oxygen mole fraction of about 0.39. *(NASA states; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 23)*
- Cool flames arose from hot flames: of 280 hot flames, 189 went out by radiative extinction and 15 reignited as cool flames, which were identified from radiant heat flux. *(NASA states; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 44)*
- A platinum (type-B) thermocouple near the flame catalysed low-temperature reactions, so hot flames that quenched reignited repeatedly through low-temperature combustion. *(NASA states; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 42)*
- Cool flames were hard to initiate and to discern during operations, and scripted tests were sometimes terminated before anyone knew a cool flame was present. *(NASA states; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 15)*
- Inverse n-butane flames burned steadily at the higher fuel concentration but showed large radiation oscillations and self-extinction at the lower one. *(NASA states; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 65)*
- A gas flame that goes out radiatively in microgravity may not stay out: a lingering cool flame, helped by hot catalytic metal, could relight it, so suppression checks and re-ignition prevention should not rely on loss of visible flame alone. *(interpretation; PSI-159_Reports_FinalRpt_ACME.CFI-G.pdf, p. 67)*

**Why it matters for fire safety** (interpretation). The documents frame CFI-G around engine efficiency and cool-flame kinetics, not fire safety. Its fire-safety value is showing that radiatively extinguished gas flames can continue as cool flames and reignite, notably near hot platinum, in conditions mapped by fuel, oxygen, pressure and flow.

- *prevention, foundational:* "Determine the conditions for which cool diffusion flames occur for propane, n-butane, and n-pentane; normal and inverse flames"
- *the brief also says:* "Microgravity tests with the highest oxygen concentrations were conducted as part of the ACME's Cool Flames Investigation with Gases (CFI-G)."

## ACME/Flame Design (PSI-10)

**Advanced Combustion via Microgravity Experiments / Flame Design**. Advanced Combustion via Microgravity Experiments (ACME), Combustion Integrated Rack (CIR), Reconfiguration of Combustion Integrated Rack (CIR). Family: ACME burner flames in the CIR. DOI 10.60555/w4zy-bt45. Licence not stated in NASA's record.

**Objective (NASA).** The primary objectives of this flight experiment are: A. Obtain soot-inception limits of quasi-steady normal and inverse spherical diffusion flames as functions of flow rate, Zst, amount of inert (N2 or CO2), and pressure for C2H4 and CH4 flames. Identify the corresponding temperatures in hot regions. Obtain similar limits and temperatures for inverse and normal coflow flames. B. Obtain detailed measurements in quasi-steady normal and inverse spherical diffusion flames for C2H4 and CH4. Determine the effects of flow rate, Zst, amount of N2, and pressure on flame temperature, size, color, and soot volume fraction. Evaluate the possible existence of steady flames. C. Obtain extinction limits as functions of Zst, amount of N2, and pressure for normal and inverse spherical C2H4 and CH4 flames. Identify the corresponding temperatures in hot regions. Identify the presence of radiative or kinetic extinction. Evaluate whether pseudo-flammability limits can be obtained for these flames.

**Approach (NASA).** Perform spherical diffusion flame tests in drop towers and aboard the International Space Station. Consider a broad range of reactant dilutions, flow rates and pressures for normal and inverse convection directions. Observe the flames with a broad suite of cameras, radiometers, photomultiplier tubes, and thermocouples.

**Impacts, on Earth and in space (NASA).** This project has the potential for improving efficiency and reducing pollutant emissions in terrestrial combustors. For example, it is contributing to novel designs of future coal power plants.

| | |
|---|---|
| Fuels or materials (rows) | ethylene (307), propane (20) |
| Phase and configuration | gas; spherical_porous_burner |
| Sizes | burner diameter 6.35 to 6.35 mm |
| Ambient oxygen | 0.20 to 0.40 (207 rows) |
| Oxygen in the injected stream | 0.21 to 0.85 (109 rows) |
| Pressure | 43 to 305 kPa (327 rows) |
| Imposed flow | none recorded |
| What varied between tests | pressure, x fuel ambient, x o2, name, flame orientation, fuel mass flow, inert mass flow, oxidizer mass flow |
| Catalogue rows | 327 tests (every row of NASA's experiment table) |
| Files at NASA | 3,807 files, 2,180.4 GB; test ids in file names: 323 |
| Measurements per test | sensor series 317, images 315, video 317, processed data 0 |
| Crew atmospheres | earth iss: 64 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | transfer tested: the CFI-G models, unchanged, refuse ethylene; with ethane as a surrogate the normal-flame model beats the incidence baseline (0.120 against 0.247), the inverse model is undecided (PROTOCOL 11.3) |
| Closest investigations | ACME CFI-G (same hardware family, same fuel or material: propane, same need: prevention); ACME s-Flame (same hardware family, same fuel or material: ethylene, same need: prevention); ACME BRE (same hardware family, same fuel or material: ethylene, same need: prevention) |

**How the tests differ.** Flames burned on a 6.35 mm porous spherical burner. Round 1 (2019) gave normal flames: ethylene, pure or nitrogen-diluted, flowing into oxygen-nitrogen ambients of 21 to 40% oxygen. Round 2 (2021-22) began with one normal propane day, then inverse flames: oxygen-nitrogen flowing into an ethylene ambient. Tests varied flow rate (some ramped), where the nitrogen was placed and so the stoichiometric mixture fraction, and pressure (logs mention 0.5 to 1.3 atm). CO2 diluent and methane were not used. Precursor tests used 2.2 and 5.2 s drop towers.

**Instruments.** ACME color digital camera (DataCam; clear, BG7 and CH* bandpass filters) (Color flame images (size, color, luminosity) at 12 bit and up to 30 fps; the BG7 filter had a thickness problem); HiBMs camera with tunable filter (Spectrally filtered flame images); Second HiBMs camera with red filter and backlight (Soot extinction (soot volume fraction), archived at 15 fps because of data limits); SiC thin filament pyrometry fiber array (14-micron fibers spaced 5 mm) (Gas temperature profile through the flame from fiber glow); CFI intensified camera with CH* filter (CH* chemiluminescence images, partly obstructed by the pyrometry arms); Thermopile radiometer arrays and off-axis fast radiometer (Flame radiant emission; the standard array saturated in most of round 1 and the CFI-G array was used in round 2); Three photomultiplier tubes (OH*, CH*, neutral density) with adjustable gain (OH* and CH* chemiluminescence and broadband emission); OH* photomultiplier as ignition detector (Ignition detection (igniter glow could cause false detection)); Thermocouple rakes (types B and K; near rake 13-100 mm, far rake 45-100 mm) and two 200 mm thermocouples (Gas temperature at set distances from the burner); Type-K thermocouple embedded in the porous spherical burner (Burner temperature); Three mass flow controllers (Fuel, oxidizer and nitrogen flow rates); CIR gas chromatograph (GCIP) (Chamber gas composition; its use limited the number of tests); SAMS accelerometer (Acceleration environment; data are at the PIMS site, not in PSI).

**Findings, from NASA's documents.**

- Across normal and inverse flames, radiative extinction occurred at a common critical peak flame temperature near 1130 K and a fixed reactant mass flux, matching the investigation's second hypothesis. *(NASA states; PSI-10_Reports_FinalRpt_ACMEFlameDesign.pdf, p. 25)*
- The ISS tests showed that burner-stabilized spherical diffusion flames can reach a steady state, with a characteristic time to steady state of about 100,000 s. *(NASA states; PSI-10_Reports_FinalRpt_ACMEFlameDesign.pdf, p. 52)*
- Long-duration flames grow, heat the burner, and can show oscillatory partial extinction. *(NASA states; PSI-10_Reports_FinalRpt_ACMEFlameDesign.pdf, p. 26)*
- Sooting depended strongly on oxygen: constantly sooting flames appeared only at 40% oxygen with high fuel concentration, while small and large flames in 21% oxygen did not soot. *(NASA states; PSI-10_Reports_FinalRpt_ACMEFlameDesign.pdf, p. 41)*
- Drop-tower work behind the flight design found that flame structure, set by moving inert between oxidizer and fuel (stoichiometric mixture fraction), dominates soot behaviour over the direction of convection. *(NASA states; PSI-10_Science Documents_SRD_ACMEFlameDesign.pdf, p. 14)*
- Because weak microgravity flames go out only near 1100-1130 K, below normal-gravity extinction temperatures, a weak flame in orbit may keep burning where 1-g experience would predict extinction. *(interpretation; PSI-10_Science Documents_SRD_ACMEFlameDesign.pdf, p. 21)*

**Why it matters for fire safety** (interpretation). NASA states ACME's gas-flame experiments target energy and environmental goals, not spacecraft fire safety. Indirectly, the results show that weak spherical microgravity flames can burn steadily for minutes and extinguish only near 1130 K, and that sooting appeared mainly at 40% oxygen, which bears on flammability limits and smoke signatures in oxygen-enriched atmospheres.

- *prevention, foundational:* "Obtain extinction limits as functions of Zst, amount of N2, and pressure for normal and inverse spherical C2H4 and CH4 flames. [...] Evaluate whether pseudo-flammability limits can be obtained for these flames."

## ACME s-Flame (PSI-23)

**Advanced Combustion via Microgravity Experiments-Structure and Response of Spherical Diffusion Flames**. Combustion Integrated Rack (CIR), ACME. Family: ACME burner flames in the CIR. DOI 10.60555/k57n-p030. Licence CC0-1.0.

**Objective (NASA).** A series of experiments is proposed with the following specific objectives, in both soot-free and sooty flames, to compare with detailed computational simulation to assess transport properties and detailed chemical kinetics (including chemiluminescence): Objective A. Measure and characterize the transient structure (such as temperature and species distribution) of diffusion flames established at initially non-steady, spherical flame structures for mixtures of different H2/CH4/diluent ratios (soot-free) and different C2H4/diluent ratios (sooty), for different flow rates. Objective B. Determine the quasi-steady convective/chemical-kinetic extinction limits (low system Damköhler numbers) and radiative/chemical-kinetic extinction limits (high system Damköhler numbers) for mixtures of different H2/CH4/diluent ratios (soot-free) and different C2H4/diluent ratios (sooty). Objective C. Determine the existence (for different Zeldovich number, Ze), onset (as a function Da), and nature (i.e. mode, frequency, amplitude) of pulsating instabilities theoretically predicted to occur in transient spherical diffusion flames using fuel/diluent mixtures that are above a critical Lewis number.

**Approach (NASA).** A series of experiments is proposed in which mixtures of various gaseous fuels and diluents will be issued into an oxidizing environment through a spherical porous burner and ignited to obtain, nominally, a spherical, one-dimensional stabilized flame. Synchronized measurements are to be made including spectrally resolved images, flame and burner temperatures, radiative flux measurements, and system operating conditions (including reactant flow rates, ambient temperatures, pressures and acceleration levels, etc.). These data are to be used for comparisons of flame behavior and structure with detailed numerical simulations of the flames. Conventional gaseous fuels including hydrogen, methane, and ethylene will be used. Hydrogen is the simplest fuel and its reaction mechanism is also reasonably established. Methane is the simplest hydrocarbon and extensive studies have been conducted on its combustion chemistry. Ethylene is a typical sooty fuel and is also an important intermediate for the oxidation of higher hydrocarbons. Diluents mixed with fuel are to include nitrogen and helium to adjust mixture diffusivities, and carbon dioxide to influence flame radiation characteristics.

**Impacts, on Earth and in space (NASA).** The impact of this work consists of both practical and fundamental aspects. Practically, with diffusion flame burning as the basis in many combustion engines and devices, understanding the underlying dynamics and chemistry of diffusion flames is important in improving performance in terms of power, efficiency, pollutant emissions, and reliability. Fundamentally, the unique buoyancy-free environment offered by microgravity affords simple aerodynamic flow fields to acquire diffusion flame data of high fidelity, with the prospect of understanding unit thermochemical processes as well as quantifying the associated physico-chemical parameters.

| | |
|---|---|
| Fuels or materials (rows) | ethylene (6), hydrogen+methane (5), hydrogen (4), methane (3) |
| Phase and configuration | gas; spherical_porous_burner |
| Sizes | burner diameter 6.35 to 6.35 mm |
| Ambient oxygen | 0.21 to 0.30 (13 rows) |
| Pressure | 101 to 101 kPa (18 rows) |
| Imposed flow | none recorded |
| What varied between tests | helium, nitrogen, species, x o2, test type, name, x fuel injected |
| Catalogue rows | 18 test days (every row of NASA's experiment table) |
| Files at NASA | 4,165 files, 782.2 GB; test ids in file names: 291 |
| Measurements per test | sensor series 286, images 289, video 285, processed data 0 |
| Crew atmospheres | earth iss: 10 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (foundational); suppression (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model; same burner as CFI-G, other fuels |
| Closest investigations | ACME/Flame Design (same hardware family, same fuel or material: ethylene, same need: prevention); ACME BRE (same hardware family, same fuel or material: ethylene, methane, same need: prevention); ACME CLD Flame (same hardware family, same fuel or material: ethylene, methane, same need: suppression) |

**How the tests differ.** s-Flame issued gaseous fuel through a 6.35 mm porous spherical burner into a quiescent oxidizer in ISS microgravity. Fuels were hydrogen, methane, hydrogen/methane blends and ethylene, diluted with nitrogen in round 1 (Sept 2019-Jan 2020) and with helium in round 2 (May-July 2020); ambients were about 21% or 30% oxygen. Fuel flow (planned 1 to 15 cc/s) was held, stepped or ramped up and down to reach radiative and kinetic extinction. CO2 dilution was left for a possible third round.

**Instruments.** Colour DataCam (up to 30 fps, 1360 x 1024) with colour, CH* and pyrometry filters (Flame colour images, CH* emission and pyrometry); Thin-filament pyrometry array (five SiC fibers, 14 micron, 5 mm spacing) (Gas temperature profile through the flame); HiBMs camera with liquid crystal tunable filter, and a second HiBMs for extinction imaging (Multispectral pyrometry; soot by light extinction); LLL-UV intensified camera with OH* filter (OH* chemiluminescence imaging (used only in s-Flame)); CFI intensified camera with CH* filter (CH* chemiluminescence imaging); Standard thermopile radiometer array (one wide-view and four narrow-view radiometers) (Thermal radiation from the flame); Three photomultiplier tubes (230-700 nm, OH*, CH*, neutral density) (Flame emission, used to detect quenching); Far-field thermocouples: short rake (type B/K, tips at 45, 60, 100, 100 mm) and two at 200 mm (Gas temperature around the flame); Type-K thermocouple embedded in the 6.35 mm porous spherical burner (Burner temperature); Three mass flow controllers (fuel up to 2 slpm) (Fuel and gas flow rates); Chamber pressure transducer, gas chromatograph and SAMS accelerometer (Chamber pressure, chamber gas composition and acceleration).

**Findings, from NASA's documents.**

- Both kinetic extinction (low Damkohler number) and radiative extinction (high Damkohler number) were observed in nitrogen-diluted methane and ethylene spherical flames, the dual extinction limits the experiment was built to map. *(NASA states; PSI-23_Reports_s_Flame_final_report_20210729.pdf, p. 30)*
- The investigators' framework treats diffusion-flame extinction as reactant leakage through the flame; other losses such as radiation act by lowering the flame temperature and so raising leakage. *(NASA states; PSI-23_Science Documents_s-Flame SCR 200802.pdf, p. 11)*
- Nitrogen-diluted hydrogen flames showed kinetic extinction when the fuel flow was ramped down but no radiative extinction. *(NASA states; PSI-23_Reports_s_Flame_final_report_20210729.pdf, p. 70)*
- Switching the diluent from nitrogen to helium shifted the extinction limits and made the spherical flames harder to ignite. *(NASA states; PSI-23_Reports_s_Flame_final_report_20210729.pdf, p. 30)*
- Of the planned regimes, transient spherical flames were obtained but no characteristically sooty ones, and the unstable behaviour seen was near the burner tube rather than the predicted pulsating instability. *(NASA states; PSI-23_Reports_s_Flame_final_report_20210729.pdf, p. 31)*
- Hardware limited the data: at high flow the porous-sphere flames became non-spherical and offset, and a leak of the first fuel bottle meant a planned H2/CH4/N2 blend was not tested. *(NASA states; PSI-23_Reports_s_Flame_final_report_20210729.pdf, p. 17)*

**Why it matters for fire safety** (interpretation). NASA's ACME documents give energy and emissions as the main aim and say it is not aimed at spacecraft fire safety, though one overview slide cites fire safety and the s-Flame final report lists extinction understanding with fire-safety implications. Its value here is foundational: kinetic and radiative extinction limits of gas flames in still microgravity, and how diluents shift them.

- *prevention, foundational:* "Determine the quasi-steady convective/chemical-kinetic extinction limits (low system Damköhler numbers) and radiative/chemical-kinetic extinction limits (high system Damköhler numbers)"
- *suppression, foundational:* "extinction limits ... for mixtures of different H2/CH4/diluent ratios (soot-free) and different C2H4/diluent ratios (sooty)"

## ACME BRE (PSI-20)

**Advanced Combustion via Microgravity Experiments - Burning Rate Emulator**. Combustion Integrated Rack (CIR), ACME. Family: ACME burner flames in the CIR. DOI 10.60555/vb93-3y72. Licence CC0-1.0.

**Objective (NASA).** The BRE flight program seeks to establish the burning conditions in a quiescent microgravity ambient in terms of material properties. The specific flight objectives are as follows. 1. Observe burning behavior in long-duration microgravity for CH4/N2 and C2H4/N2mixtures on porous circular burners of 25 and 50 mm diameter. Consider O2/N2 ambients with various pressures and ambient O2 concentrations and pressures. NASA - considered atmosphere for human space missions will be examined: 14.7 psia, 21 % oxygen: 10.2 psia, 26.5 %; and 8.2 psia, 34 % oxygen. Emphasize fuel conditions that can be related to a diverse range of condensed fuels. 2. Consider the conditions of Objective 1 for ambients of O2/CO2 (desired). 3. Consider the conditions of Objectives 1 – 2 for extinction limits.

**Approach (NASA).** To accomplish the above objectives, flight experiments are proposed. Additionally, preliminary flight tests will be performed to verify the experiment operations and for comparison with ground-based tests. For each test, conditions will be established for the chamber contents, namely choice of diluent, oxygen concentration, and pressure. Fuel flow will commence followed by ignition. The flames will initially be observed without varying the fuel flow rate. Because burner surface temperature and heat flux will vary, this will allow a broad range of conditions to be observed during a single test. As the surface temperature of the burner increases, and the flame remains steady, a continuous set of data is produced for the flammability map. Eventually either the flame will extinguish, the surface temperature of the burner will equilibrate, or the temperature limit of the heat flux sensor will be reached. At this time the fuel flow will be terminated. So a test should terminate for a given flow rate at extinction, and for continued burning when the burner surface temperature has steadied or the temperature limit of the heat flux sensor is neared. The extinction limit could be found once the burner temperature has steadied, and the heat flux sensor has not exceeded its limit. At that state, the flow rate can be reduced in small increments, until extinction is observed and the flow rate shut off. Again, in this process, new data is being produced for the flammability map. Ignition limit tests are not proposed for microgravity, but we will explore these under normal gravity. We believe there is better control on Earth-based ignition testing and that these results should carry over to microgravity conditions. In addition to the flight experiments, in normal gravity testing will continue. These tests will involve gaseous, liquid, and solid fuels. These experiments will further establish the validation of the BRE for 1-g applications and show its usefulness to establish 1-g ignition and extinction limits. As available, drop facility testing will be used to refine procedures, test instrumentation, and help to establish viable flow conditions. We will also pursue a CFD model for the quiescent microgravity burning. This will also guide us in the flight experiments, and potentially provide a predictive methodology.

**Impacts, on Earth and in space (NASA).** BRE yielded important data relevant to spacecraft fire safety. Quasi-steady burning corresponding to various condensed-phase materials (such as PMMA and Nylon) was observed in microgravity despite the absence of air flow, where the cessation of ventilation is an established step in putting out spacecraft fires. The ongoing combustion without air flow was notably seen in elevated-oxygen atmospheres which are being considered for use in future spacecraft, whereas such flames in a normal atmosphere of 21% oxygen self-extinguished in a short time. While BRE’s results are still preliminary, they suggest that NASA may want to increase use of its alternative flammability Test 2, with the cone calorimeter, where that test can measure material properties corresponding to the microgravity flames observed with the BRE burner.

| | |
|---|---|
| Fuels or materials (rows) | ethylene (16), methane (2) |
| Phase and configuration | gas; flat_porous_burner |
| Sizes | burner diameter 25 to 50 mm |
| Ambient oxygen | 0.21 to 0.34 (2 rows) |
| Pressure | not in the table |
| Imposed flow | none recorded |
| What varied between tests | x o2, name, burner diameter, x fuel injected |
| Catalogue rows | 18 test days (every row of NASA's experiment table) |
| Files at NASA | 2,361 files, 2,035.6 GB; test ids in file names: 208 |
| Measurements per test | sensor series 187, images 207, video 189, processed data 0 |
| Crew atmospheres | earth iss: 0 inside, 1 oxygen only, stated by NASA; moon base rover: 0 inside, 1 oxygen only, stated by NASA; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model |
| Closest investigations | ACME/Flame Design (same hardware family, same fuel or material: ethylene, same need: prevention); ACME s-Flame (same hardware family, same fuel or material: ethylene, methane, same need: prevention); ACME CFI-G (same hardware family, same need: prevention) |

**How the tests differ.** Every test burned gaseous fuel on a flat porous burner in a quiescent chamber, with no imposed flow. Tests varied burner diameter (25 mm in round 1; 25 and 50 mm in round 2), fuel (ethylene, pure or 50% in nitrogen, and methane on the 25 mm burner), fuel mass flux (about 1 to 12 g/m2s), oxygen in nitrogen (0.21 to 0.40) and pressure (about 0.565 to 1 bar). Planned O2/CO2 ambients were not run. Normal-gravity and 5 s drop tests served as comparisons.

**Instruments.** Two thermopile (Medtherm) heat-flux gauges in the burner surface, at the centre and offset (net heat flux from the flame to the burner surface, which with the fuel mass flux gives the heat of gasification); Copper calorimeter (the burner's copper top) (average surface heat flux from the copper temperature rise); K-type thermocouples in the burner (burner temperatures used in the calorimeter energy balance and surface re-radiation); Thermopile radiometer arrays (BRE and standard arrays) (flame radiant emission and the flame radiation loss fraction); Mass flow controllers (Hastings) (fuel flow, converted to fuel mass flux); ACME DataCam, 12-bit colour camera with filter wheel (BG7, CH* 430 nm) (flame images (size, shape, colour) at up to 30 fps); Thin filament pyrometry with 14-micron SiC fibres (gas temperature from the colour of fibres glowing in the flame); Three photomultiplier tubes (230-700 nm, OH* and CH* filters) (flame emission and chemiluminescence); CFI intensified CH* camera (CH* chemiluminescence images of the flame (partly obstructed by the pyrometry arms)); Thermocouple rakes (types B and K) and thermocouples at 200 mm (gas temperatures away from the burner); CIR gas chromatograph (chamber gas composition before and after tests (rarely used for BRE)); CIR pressure transducers and chamber calculator (chamber pressure and estimated chamber gas composition); SAMS accelerometers (acceleration environment (not archived in PSI)).

**Findings, from NASA's documents.**

- Quasi-steady burning of a flat surface in a quiescent microgravity atmosphere was achieved at oxygen mole fractions of 0.265 and above, and pressure had only a weak effect. *(NASA states; PSI-20_Reports_Final_20220201.pdf, p. 39)*
- Flames emulating condensed materials such as PMMA and Nylon kept burning without any air flow, notably in the elevated-oxygen atmospheres considered for future spacecraft, while in 21% oxygen they self-extinguished in a short time. *(NASA states; PSI metadata record, field research_impacts)*
- Extinction is tied to a critical flame temperature of about 1100 to 1200 K, which bounds the microgravity flammability maps with two limits: radiative extinction of large flames and heat-loss extinction of small flames. *(NASA states; PSI-20_Reports_Dissertation-PD-revised_.pdf, p. 109)*
- The flames were blue and soot-free, so their radiation came from gaseous products; steady flames radiated no more than about 60% of their energy, and the radiant fraction correlated with flame height. *(NASA states; PSI-20_Reports_Dissertation-PD-revised_.pdf, p. 45)*
- The flammability map indicates that 25 mm surfaces with heat of combustion 23.6 to 47.2 kJ/g and heat of gasification 1 to 10 kJ/g can burn steadily; lower fuel fraction and higher oxygen favour longer burns. *(NASA states; PSI-20_Reports_Dissertation-PD-revised_.pdf, p. 54)*
- Interpreted for real materials, plastics burn more easily but more slowly in microgravity: PE and PP near 3 g/m2s, nylon and PMMA near 2.5 g/m2s, far below their normal-gravity burning fluxes. *(NASA states; PSI-20_Reports_Dissertation-PD-revised_.pdf, p. 51)*

**Why it matters for fire safety** (NASA states). Stopping ventilation, an established step in putting out spacecraft fires, may not extinguish a fire in the elevated-oxygen atmospheres considered for future spacecraft, where emulated PMMA and Nylon flames kept burning. NASA suggests wider use of flammability Test 2 (cone calorimeter), which measures the material properties that correspond to these microgravity flames.

- *prevention, named in the challenge brief:* "While it burned gaseous fuels, the Burning Rate Emulator (BRE) experiment of the Advanced Combustion via Microgravity Experiments (ACME) project was a study of materials flammability in still atmospheres as a function of material properties."

## ACME CLD Flame (PSI-21)

**Advanced Combustion via Microgravity Experiments / Coflow Laminar Diffusion Flame**. Combustion Integrated Rack (CIR), ACME. Family: ACME burner flames in the CIR. DOI 10.60555/23e1-9950. Licence CC0-1.0.

**Objective (NASA).** In order to provide the experimental data needed to allow rigorous testing of computational models at the extremes of the fuel dilution spectrum, we have the following objectives: A) Characterize the behavior of weak coflow diffusion flames of methane and ethylene, as a function of velocity and fuel dilution, through measurement of flame shape, lift-off heights, temperatures and extinction limits. B) Characterize the sooting tendencies of diffusion flames with long residence times, as a function of velocity and fuel dilution, through measurement of flame shape, gas temperatures in the soot-free portion of the flame, soot luminosity, soot temperatures, and soot volume fractions.

**Approach (NASA).** ACME CLD Flame project goals include the generation of improved computational models with improved soot sub-models that are effective for both dilute flames and sooting flames that vary from low to high soot loading levels. The enhanced time and spatial scales in a microgravity environment will be important for achieving both goals. Experimentally, we plan to study methane and ethylene diffusion flames over a wide range of test conditions. For the more dilute flames, measurements will be made of flame chemiluminesence of OH* and CH* to observe variations in lift-off height and flame shape as extinction limits are approached. The sooting tendencies of the more fuel-rich flames will be investigated using soot luminosity and laser extinction measurements to determine soot temperatures and soot volume fractions. Flame temperatures in non-sooting regions will be measured using thin-filament pyrometry. Based on observations, modifications can be made to the kinetics and soot model to match better the experimental results. If the overall agreement of the measurements and computations is good, the results of the computations can be “mined” to provide insight on species (and hence chemical processes) that cannot be measured directly using the diagnostics available for the proposed experiment.

**Impacts, on Earth and in space (NASA).** Diffusion flames are the flame type of most practical combustion devices. The ability to predict the coupled effects of complex transport phenomena with detailed chemical kinetics in such systems is critical in the modeling of turbulent reacting flows and in understanding the processes by which soot formation and radiative transfer take place. In addition, an understanding of those factors that affect flame extinction in diffusion flames is critical in the suppression of fires and in improving engine efficiency. While normal gravity combustion studies can provide important information on combustion processes, the effects of gravitational forces often complicate the interpretation of both computational and experimental results. Momentum effects dominate most practical combustion devices, but the contribution of buoyancy forces is hard to eliminate in normal gravity. A goal of the study will be to apply sophisticated numerical tools and quantitative diagnostic techniques to the study of hydrocarbon diffusion flames in a microgravity environment. This will lead to a more detailed understanding of the interaction of convection, diffusion and chemistry under momentum-dominated conditions.

| | |
|---|---|
| Fuels or materials (rows) | methane (16), ethylene (16) |
| Phase and configuration | gas; coflow_burner |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | stated by NASA, not per test |
| What varied between tests | fuel mfc full scale, n2 mfc full scale, test type, name, x fuel injected |
| Catalogue rows | 32 test days (every row of NASA's experiment table) |
| Files at NASA | 4,134 files, 1,998.3 GB; test ids in file names: 359 |
| Measurements per test | sensor series 352, images 324, video 350, processed data 0 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | suppression (foundational) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model |
| Closest investigations | ACME s-Flame (same hardware family, same fuel or material: ethylene, methane, same need: suppression); ACME/Flame Design (same hardware family, same fuel or material: ethylene); ACME BRE (same hardware family, same fuel or material: ethylene, methane) |

**How the tests differ.** Over 32 test days (day codes 17319 to 21062), CLD Flame burned coflow diffusion flames of methane and of ethylene, plus a set of improved coflow tests. Each day a fuel bottle (for example 40% methane) was diluted with nitrogen over a stated concentration range, such as 30-40%, with 2 to 18 flames ignited; fuel and nitrogen flow controller sizes changed between days. NASA's objectives name flow velocity and fuel dilution as the parameters varied.

**Instruments.** OH* and CH* chemiluminescence imaging (Lift-off height and flame shape of dilute flames as extinction limits are approached (as planned in NASA's approach text)); Soot luminosity imaging (Soot temperatures of the more fuel-rich, sooting flames (planned)); Laser extinction (Soot volume fractions of the sooting flames (planned)); Thin-filament pyrometry (Flame temperatures in non-sooting regions (planned)); Fuel and nitrogen mass flow controllers (Fuel and nitrogen flows that set the fuel concentration on each test day).

**Findings, from NASA's documents.**

- CLD Flame was designed as a benchmark for computational combustion models at both ends of the fuel dilution range: weak, dilute flames near extinction and long-residence-time sooting flames. *(NASA states; PSI metadata record, field objective)*
- Microgravity is used to obtain momentum-dominated flames, as in practical combustion devices, without the buoyancy that complicates normal-gravity results, and to gain longer time and larger spatial scales. *(NASA states; PSI metadata record, field research_impacts)*
- NASA links the extinction work to fire safety: knowing which factors drive extinction of diffusion flames is critical for suppressing fires. *(NASA states; PSI metadata record, field research_impacts)*
- For dilute flames, NASA's approach treats changes in lift-off height and flame shape, seen in OH* and CH* emission, as the signature of an approaching extinction limit. *(NASA states; PSI metadata record, field approach)*
- The test-day table records the fuel concentration range tried, not measured limits: extinction can come before the lowest listed concentration, so the table alone does not give extinction limits. *(interpretation; PSI-21_Experimental table_CLD Flame.csv, p. 1)*

**Why it matters for fire safety** (interpretation). NASA states that understanding what drives extinction of diffusion flames is critical for suppressing fires. CLD Flame contributes benchmark data on weak, near-extinction and sooting coflow flames in microgravity; links to spacecraft fires are indirect, since it burns methane and ethylene from a burner, not spacecraft materials.

- *suppression, foundational:* "Characterize the behavior of weak coflow diffusion flames of methane and ethylene, as a function of velocity and fuel dilution, through measurement of flame shape, lift-off heights, temperatures and extinction limits."

## ACME E-FIELD Flames (PSI-22)

**Advanced Combustion via Microgravity Experiments Electric-Field Effects on Laminar Diffusion Flames**. Combustion Integrated Rack (CIR), ACME. Family: ACME burner flames in the CIR. DOI 10.60555/se5a-8c73. Licence CC0-1.0.

**Objective (NASA).** The objective of this investigation is to better understand the relationship between electric field voltage and chemi-ion current, analyze the dynamic response of flames to electric field changes, evaluate soot formation, and characterize stabilized near-limit lifted flames.

**Approach (NASA).** The E-FIELD flames experiment investigates how electric fields influence the behavior of laminar diffusion flames in microgravity. This unique environment is ideal for isolating electric field effects from buoyancy forces. The experiment is conducted with gas-fueled flames, both with and without a coflow of air. In the former case, the tests are conducted using the same coflow burner used by the Coflow Laminar Diffusion (CLD) Flame experiment. An electric field is generated by creating a high voltage (up to 10 kV) differential between the burner and a flat circular mesh suspended above (i.e., downstream of) the burner. Measurements, as a function of the field strength and fuel, are taken of the ion current through the flame and the flame’s response time to electric forcing. The gas-jet burner is used to create a small, focused flame for detailed ion-driven convection studies while the coflow burner surrounds the fuel jet with a weak synthetic air flow to enable flame stability and soot behavior analysis. A high-voltage copper mesh electrode positioned above the burner tip generated an electric field with controlled polarity and strength. The electric field interacts with ions generated through chemi-ionization during combustion, driving ion-driving convection and influencing flame behavior. Methane and ethylene were used as fuels and diluted with nitrogen to control flame strength. Dilution levels were varied to explore a range of combustion dynamics, from highly stable to near-extinction flames. Voltage-current curves were recorded to assess the relationship between electric field strength and chemi-ion current, revealing ion-driven convection effects. High-resolution cameras captured flame shape, luminosity, and soot distribution. Spectral imaging with filters measures chemiluminescent emissions and soot volume fraction. There were thermopile detectors to measure flame radiant emissions. All experiments were conducted in the combustion integrated rack (CIR). Tests were administered with Glenn Research Center (GRC) directing astronaut actions. Parameters such as fuel flow rate, coflow velocity, and field polarity are were varied to isolate specific effects on ion production, flame stability, and soot behavior.

**Impacts, on Earth and in space (NASA).** The E-FIELD Flames research is not being conducted to serve any space applications, but it is possible that its findings could aid the development of future space-based combustion devices (e.g., for solid waste processing). The use of electric fields to control flames can reduce the production of harmful pollutants in practical terrestrial combustion. The combustion of fossil fuel is the primary human source of greenhouse gas and acid rain and furthermore creates soot, which can be beneficial but is also a health hazard. Combustion using electrical fields may also allow for more efficient combustion, reducing fuel consumption in addition to unwanted emissions.

| | |
|---|---|
| Fuels or materials (rows) | methane (11), ethylene (9) |
| Phase and configuration | gas; coflow_burner, gas_jet_burner |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | none recorded |
| What varied between tests | mesh polarity, test type, direction, name, configuration, x fuel injected |
| Catalogue rows | 20 test days (every row of NASA's experiment table) |
| Files at NASA | 2,578 files, 950.1 GB; test ids in file names: 225 |
| Measurements per test | sensor series 204, images 219, video 215, processed data 0 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need |  |
| Towards partial gravity | mimics it with an electric field |
| Hand-written models | none: no model |
| Closest investigations | ACME/Flame Design (same hardware family, same fuel or material: ethylene); ACME s-Flame (same hardware family, same fuel or material: ethylene, methane); ACME BRE (same hardware family, same fuel or material: ethylene, methane) |

**How the tests differ.** Two 2018 ISS rounds used different burners: a coflow burner (round 1) and a 2.1 mm gas-jet burner (round 2). Across tests NASA varied the fuel (methane or ethylene, pure or nitrogen-diluted from 30 to 100%), fuel velocity, whether a 21% oxygen coflow was supplied, mesh electrode polarity, and the applied voltage, swept in steps up to 10 kV. First-phase tests ran at 100 kPa. Precursor 1-g and 2.2 s drop-tower tests compared gravity levels.

**Instruments.** Copper mesh electrode with high-voltage supply and shunt resistor (Ion (chemi-ion) current through the flame, from the voltage across a known resistor, as the mesh voltage is changed up to 10,000 V of either polarity); E-field high voltage monitor channel (Applied mesh voltage, logged with the non-imaging data); ACME color digital camera (DataCam) with filters (Flame shape, luminosity and soot pyrometry; low sensitivity forced long exposures and the blue-green filter reduced pyrometry accuracy); CFI intensified monochrome camera with CH* filter (CH* chemiluminescence images of the flame); Three photomultiplier tubes (OH*, CH*, neutral density) (Filtered chemiluminescent and broadband flame emission); Radiometers and PMTs as event detectors (Detection of ignition and extinction, with non-imaging data recorded at 125 Hz); Thermopile radiometers (six) (Flame radiant emission); HiBMs camera with 650-655 nm laser-diode backlight (Soot volume fraction by light extinction; largely blocked by the electrode mesh mount); Mass flow controllers (Oxidizer, nitrogen bypass and fuel flow rates); Pressure transducers (Outer coflow, inner coflow/burner and chamber pressure); Type-K thermocouple in the gas-jet burner (Fuel tube temperature 5 mm upstream of the outlet (the coflow burner had none)); SAMS accelerometer (Acceleration environment; SAMS data are not in the PSI archive).

**Findings, from NASA's documents.**

- The experiment rests on chemi-ions made in hydrocarbon flames: an applied field drives them, producing an ion-driven wind that acts as a body force on the gas and can be used to measure and reshape a flame without buoyancy. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 4)*
- Long-duration microgravity was needed to separate the ion wind from buoyancy: the 2.2 s drop tower is too short for flames to reach steady state, and a quasi-steady voltage-current sweep takes over 100 s. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 6)*
- In microgravity the flame's ion current rises to a distinct peak as the applied voltage increases and then drops sharply, instead of the saturation plateau seen in 1-g. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 81)*
- The applied field stabilized very dilute, near-limit flames, especially when no air coflow was supplied. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 23)*
- Field polarity sets the response: a positive field compresses the flame and its soot and led to an unpredicted oscillation before extinction, while a negative field keeps the flame near spherical. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 72)*
- Burner geometry changes the electrical response: the gas-jet burner gave a clear parabolic rise of ion current with oscillations that the coflow burner did not show. *(NASA states; PSI-22_Reports_E-FIELD Flames Final Report.pdf, p. 65)*

**Why it matters for fire safety** (interpretation). NASA states E-FIELD Flames is not conducted for space applications, and ACME's gas-flame experiments other than BRE target energy and emissions rather than spacecraft fire safety. Any fire-safety value is indirect: microgravity data on how electric fields change near-limit flame stability, soot and extinction, and on ion current as a flame-state signal.


## SPICE (PSI-107)

**Smoke Point In Co-flow Experiment**. Microgravity Science Glovebox (MSG). Family: flow duct in the MSG. DOI 10.60555/9kbh-e962. Licence CC0-1.0.

**Objective (NASA).** The objective of SPICE is to better understand the complex process of combustion by observing flames in a microgravity environment aboard the International Space Station (ISS) and analogous ground experiments. Specific objectives were: (1) Identify the laminar smoke-point properties of non-buoyant jet diffusion flames under various co-flow conditions in microgravity. (2) Determine the effects of burner diameter and co-flow velocity on the luminous flame lengths and smoke points. (3) Quantify the sooting propensity of different fuels, such as ethylene, propane, and propylene mixtures. (4) Validate the correlation between luminous flame shape and residence time under microgravity conditions. (5) Investigate the impact of air and fuel flow rate variations on smoke point phenomena in space.

**Approach (NASA).** The experimental setup in the microgravity science glovebox consisted of an apparatus with a flow duct with integrated nozzles, cameras, and fuel delivery systems. Fuels tested included methane, ethylene, propane, propylene, and propylene mixtures. Each flame was ignited in a controlled co-flow air environment, with burner diameters of 0.41 mm, 0.76 mm, and 1.6 mm and co-flow velocities ranging from 5.4 cm/s to 65 cm/s. Fuel flow rates were adjusted to identify the smoke point, marked by the flame's transition to emitting soot. Data such as flame lengths were recorded using still and video cameras. Measurements included flame height, radiative heat losses, and co-flow air velocity distributions. In total 526 flames were studied.

**Impacts, on Earth and in space (NASA).** (1) Will improve our understanding of soot formation mechanisms and help design safer spacecraft environments. (2) Will contribute to refining our computational models for predicting flame behaviors under non-buoyant conditions. (3) Will enhance our knowledge of soot generation (4) Could possibly provide insights to better fuel formulations and combustion processes.

| | |
|---|---|
| Fuels or materials (rows) | propylene (339), propane (50), ethylene (42), methane (4) |
| Phase and configuration | gas; coflow_burner |
| Sizes | nozzle inner diameter 0.41 to 1.6 mm |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | -6 to 85 cm/s (415 rows) |
| What varied between tests | fuel volume flow std, speed, name, nozzle inner diameter, fuel mass flow, total mass flow, x fuel injected |
| Catalogue rows | 435 test points (every row of NASA's experiment table) |
| Files at NASA | 188 files, 6.9 GB; test ids in file names: 53 |
| Measurements per test | sensor series 0, images 53, video 0, processed data 0 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | detection (foundational) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | none: no model |
| Closest investigations | SLICE (same hardware family, same fuel or material: ethylene, methane); BASS-II (same hardware family, same need: detection); ACME CFI-G (same fuel or material: propane) |

**How the tests differ.** Laminar jet diffusion flames burned in an air co-flow duct in the ISS glovebox. NASA varied fuel (ethylene, propane, propylene, and propylene diluted with nitrogen to 75 and 50 percent; methane gave no smoke points), burner diameter (0.41, 0.76 and 1.6 mm), co-flow air velocity (5.4 to 65 cm/s) and fuel flow rate, which was adjusted to find each smoke point. Normal-gravity tests with the SPICE module and an unconfined 13.7 mm burner gave comparison data.

**Instruments.** Nikon D100 digital still camera (high-resolution flame images from above, used for luminous flame length); Panasonic colour video camera (flame video; smoke-point length measured to the luminous tip at the maximum grayscale gradient); Thermopile radiometer (qualitative flame radiation); Onboard duct anemometer (co-flow air velocity, correlated with a second anemometer 13 mm above the burner tip); Sierra mass flow controller (fuel flow rate as nitrogen-equivalent flow, corrected with gas-specific K factors); TSI 1261A hot-wire probe (pre-flight survey) (air velocity map inside the flight flow duct at axial stations 13 and 56 mm across fan settings); Retractable 105 mm reference ruler (length scale for flame images); Hydrogen diffusion flame in the plume (ground unconfined tests) (presence of emitted soot by its blackbody glow).

**Findings, from NASA's documents.**

- Faster co-flow and smaller burners lengthen non-buoyant smoke points; the final report scales smoke-point length with burner diameter to the power -0.91 and co-flow velocity to the power 0.41. *(NASA states; PSI-107_Reports_SPICE_FinalReport.pdf, p. 24)*
- The ISS smoke points give a fuel sooting order of propane, ethylene, 50 percent propylene, 75 percent propylene and propylene. *(NASA states; PSI-107_Presentations_Laminar Smoke Points in Coflow Measured Aboard ISS.pdf, p. 23)*
- Smoke-point flame length is proportional to fuel mass flow rate, with only a second-order effect of the co-flow to fuel velocity ratio. *(NASA states; PSI-107_Reports_SPICE_30-DayPostFlightReport.pdf, p. 9)*
- A simplified residence-time analysis fails to collapse the smoke-point data, and no mechanism for the co-flow and burner effects is proven. *(NASA states; PSI-107_Reports_SMOKE POINTS OF MICROGRAVITY AND  NORMAL GRAVITY COFLOW DIFFUSION  FLAMES_DotsonThesis.pdf, p. 75)*
- In non-buoyant flames soot forms near the cool core rather than near the flame sheet as in buoyant flames, and the earlier LSP experiment showed the microgravity smoke point can be dominated by radiative loss and tip extinction. *(NASA states; PSI-107_Science Documents_SPICE_SRD.doc, p. 3)*
- Faster co-flow raises the fuel flow a flame can carry before emitting soot, so a gas-jet fire in stronger ventilation could grow larger while releasing little smoke for detectors; SPICE did not test detection. *(interpretation; PSI-107_Presentations_Laminar Smoke Points in Coflow Measured Aboard ISS.pdf, p. 23)*

**Why it matters for fire safety** (NASA states). NASA states that non-buoyant smoke-point data are needed for spacecraft fire safety and frames SPICE around soot generation and oxidation in such flames. The documents do not say whether the results serve detection, prevention or suppression.

- *detection, foundational:* "Identify the laminar smoke-point properties of non-buoyant jet diffusion flames under various co-flow conditions in microgravity."

## SLICE (PSI-106)

**Structure and Liftoff In Combustion Experiment**. . Family: flow duct in the MSG. DOI 10.60555/pm12-rg61. Licence CC0-1.0.

**Objective (NASA).** Objective: The objective of this investigation is to better understand the structure and stability of laminar diffusion flames in microgravity. Specific objectives of this investigation were: 1. To investigate the structure of laminar diffusion flames in microgravity to understand the stabilizing mechanisms at the flames base. 2. Determine the liftoff velocity limits for different fuels and burner diameters. 3. Evaluate the impact of fuel dilution with nitrogen on soot formation and flame stability. 4. Quantify the effects of microgravity on flame length and radiative heat loss compared to normal gravity conditions. 5. Validate numerical models for flame dynamics, soot production, and thermal radiation in non-premixed flames.

**Approach (NASA).** The SLICE apparatus consisted of a fan-driven flow duct, exchangeable burner tubes, and integrated sensors to measure flow rates and temperature. This setup used several fuels such as methane, ethylene, and nitrogen-diluted mixtures. Flames were then manually ignited and observed under controlled coflow air conditions. A high resolution camera captured still images while sensor data was overlaid on these images. Calibration tests for airflow velocity, soot measurement, and the fans were conducted to ensure the accuracy of the results. More than 4,000 high-resolution images were captured across 13 testing experiments. Data was then analyzed to correlate flame structure with stability with flow dynamics and fuel properties.

**Impacts, on Earth and in space (NASA).** 1. Improved numerical models for flame behavior lead to more efficient combustion technologies on Earth. 2. Understanding soot formation aids in designing combustion systems with minimum emissions. 3. Insights into flame stability and behavior to enhance fire safety. 4. Findings will inform the development of cleaner, more efficient engines and industrial burners.

| | |
|---|---|
| Fuels or materials (rows) | methane (8), ethylene (2), ethane (1), None (1) |
| Phase and configuration | gas; coflow_burner |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | stated by NASA, not per test |
| What varied between tests | name, x fuel injected |
| Catalogue rows | 12 test days (every row of NASA's experiment table) |
| Files at NASA | 18,707 files, 60.8 GB; test ids in file names: 0 |
| Measurements per test | sensor series 0, images 17, video 6, processed data 13 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (foundational) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | none: no model |
| Closest investigations | ACME CFI-G (same fuel or material: ethane, same need: prevention); ACME/Flame Design (same fuel or material: ethylene, same need: prevention); ACME s-Flame (same fuel or material: ethylene, methane, same need: prevention) |

**How the tests differ.** Gas-jet flames were ignited manually from exchangeable burner tubes in the fan-driven coflow of air of the glovebox flow duct. Tests varied the fuel and its nitrogen dilution (pure methane, methane at 70% and 40% in nitrogen, ethylene at 20% in nitrogen), the burner diameter and the coflow velocity. The record lists no change of oxygen or pressure; some test days were pyrometry or fan calibrations.

**Instruments.** High-resolution still camera (flame images (more than 4,000) with sensor data overlaid); Integrated flow sensors (flow rates); Integrated temperature sensors (temperature); Pyrometry (temperature; only its calibration tests are logged in the record).

**Findings, from NASA's documents.**

- NASA's hypothesis is that the structure and stability of laminar diffusion flames in microgravity depend strongly on burner diameter, coflow velocity and fuel composition, and are fundamentally different from flames on Earth. *(NASA states; PSI metadata record, field hypothesis)*
- NASA lists insights into flame stability and behaviour that enhance fire safety among SLICE's impacts, alongside improved numerical flame models, soot understanding for low-emission design, and cleaner engines and industrial burners. *(NASA states; PSI metadata record, field research_impacts)*
- The test log makes nitrogen dilution of the fuel the main fuel variable, from pure methane down to 40% methane and 20% ethylene, all in air coflow; with no oxygen or pressure variation recorded, the data describe ordinary-air conditions only. *(interpretation; PSI-106_Experimental table_Slice.csv, p. 1)*
- Liftoff limits mark when a gas-jet flame detaches from its source; for spacecraft this could bear on whether a flame at a leaking gas line stays anchored in ventilation flow, but the documents given report no limits. *(interpretation; PSI metadata record, field objective)*

**Why it matters for fire safety** (NASA states). NASA names insights into flame stability and behaviour that enhance fire safety as one SLICE impact, but its other stated impacts concern combustion models, soot emissions and cleaner engines on Earth. The documents given contain no result that applies SLICE to a spacecraft fire scenario.

- *prevention, foundational:* "Determine the liftoff velocity limits for different fuels and burner diameters."

## BASS-II (PSI-25)

**Burning and Suppression of Solids - II**. Microgravity Science Glovebox (MSG). Family: flow duct in the MSG. DOI 10.60555/4qc4-de67. Licence CC0-1.0.

**Objective (NASA).** BASS-II results contribute to the combustion computational models used in the design of fire detection and suppression systems in microgravity and on Earth. Objectives Include: 1) Study the ignition, flame growth, flame spread, and extinction limits for solid fuels burning in low-velocity forced flows in microgravity. 2) Begin to bridge the gap between the normal gravity NASA-STD-6001 Test #1 method, ground-based microgravity tests, and actual material flammability in microgravity. 3) Provide SoFIE PIs with preliminary data to refine Science Requirements. 4) Practical, realistic (thicker) fuels in typical geometries will be examined, including slabs, cylinders, and spherical sections. The primary variables include: • Forced flow velocity (speed and direction) • Ambient oxygen concentration (via working volume nitrogen vitiation) • Sample geometry (rods, spherical section, slabs, films, and fabric sheets

**Approach (NASA).** To summarize, BASS-II's approach, microgravity combustion tests were performed aboard the International Space Station. Active crew support for real-time “lab partner” operations. The wind tunnel was installed in the Microgravity Science Glovebox which supplied power, imaging, and a level of containment. Fuel samples were mounted inside a small wind tunnel which could impose airflow speeds up to 40 cm/s. Main variables are airflow speed and oxygen concentration (oxygen varied below 21% by adding nitrogen). Flame appearance, flame growth, and spread rates were determined in both the opposed and concurrent flow.

**Impacts, on Earth and in space (NASA).** A primary goal of BASS-II is improved spacecraft fire safety, improved understanding of combustion in space and how to avoid it. If you’re on a mission far from Earth, a fire can be catastrophic. BASS-II helps engineers select the safest materials and improve firefighting methods on future space missions. In outer space, fuels burn as oval balls rather than with an upward pointed cone flame as they do on Earth. This simpler combustion process enables researchers to evaluate computer models of fuel burning. These models can then be used to more accurately study flames on Earth, such as in wildfires, building fires, energy recapture from waste recycling, and other combustion problems.

| | |
|---|---|
| Fuels or materials (rows) | sibal_fabric (21) |
| Phase and configuration | solid; flat_sample |
| Ambient oxygen | 0.16 to 0.21 (21 rows) |
| Pressure | not in the table |
| Imposed flow | 5 to 11 cm/s (8 rows) |
| What varied between tests | carbon dioxide, x o2, fan display, flow restrictor, x co, speed |
| Catalogue rows | 21 tests (the project's cohort: the cotton-fiberglass fabric burns, train and validation; the investigation burned other samples too) |
| Files at NASA | 695 files, 274.6 GB; test ids in file names: 115 |
| Measurements per test | sensor series 0, images 115, video 112, processed data 0 |
| Crew atmospheres | earth iss: 0 inside, 6 oxygen only; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief); suppression (stated in NASA's objective for the investigation); detection (stated in NASA's objective for the investigation) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | scored model: bass_front.wsp D1: position of the flame tip on the fabric, scored on burns measured blind (CRPS 0.117 sample widths against 0.234 for persistence; development number). Its spread-or-not boundary fails |
| Closest investigations | SPICE (same hardware family, same need: detection); SLICE (same hardware family, same need: prevention); BASS (same hardware family, same need: prevention, suppression) |

**How the tests differ.** Glovebox flow-duct tests varied the solid fuel and geometry (thin PMMA films 0.1 to 0.4 mm, PMMA slabs 1 to 5 mm burning on one or two sides, clear and black PMMA rods of three diameters, a PMMA sphere section, Nomex, a wax candle, cotton-fiberglass fabric), sample width, flow direction (opposed, concurrent, stagnation), flow speed (about 1 to 55 cm/s) and oxygen, lowered from about 22% to 13.6% by diluting the glovebox with nitrogen at cabin pressure. Ground tunnels and 5.18 s drop tests gave normal-gravity comparisons.

**Instruments.** Airspeed transducer (anemometer) in the flow duct (air flow speed over the sample); Thermopile radiometer (flame radiation signal (uncalibrated gain, recorded at 5 Hz in the video data)); Video camera and Nikon D300s digital still camera (flame appearance, size and leading-edge position, with an overlay of test data); Top-view image processing of the flame leading edge (flame spread rate from an intensity threshold); Gas sensors CDM and CSA-CP, with the ISS MCA used for an oxygen offset (O2, CO2 and CO in the glovebox before and after each burn); Post-flight weighing of samples (sample mass loss).

**Findings, from NASA's documents.**

- Over thin PMMA in opposed flow, flames underwent radiative extinction when the flow was reduced below a critical velocity of a few cm/s; this velocity does not depend on fuel thickness but rises at lower oxygen. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 36)*
- For PMMA rods, ignition limits lay above the flammability limits, and ignition delays reached up to ten times the igniter time of NASA-STD-6001. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 97)*
- Along the PMMA rod flammability boundary, analysed with a critical Damkohler number formulation, heat losses were negligible on the high-flow blowoff branch but dominated by radiation on the low-flow quenching branch. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 98)*
- Comparing ISS opposed-flow spread with normal-gravity downward spread over PMMA extended the thermal-regime limit to downward spread in air, giving an effective buoyant velocity of about 1.9 cm/s and a thin-to-thick transition thickness of about 5.2 mm. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 52)*
- Opposed-flow tests on PMMA slabs gave the first thick-fuel microgravity spread data: sample width affected spread over the thinner fuels, two-sided burning spread faster, and the FDS model overpredicted spread rates. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 71)*
- Combustion-product sensing showed CO/CO2 ratios well above those of efficient combustion and heat release rates mostly under 100 W; CO diluted into the US Lab would stay below the spacecraft maximum allowable concentration. *(NASA states; PSI-25_Reports_BASS-II Summary Report.pdf, p. 16)*

**Why it matters for fire safety** (NASA states). NASA says BASS-II helps engineers select the safest materials and improve firefighting methods, and that its results feed the models used to design fire detection and suppression systems. The tests map where solids stop burning as flow speed and oxygen fall, and show microgravity ignition delays up to ten times the NASA-STD-6001 igniter time.

- *prevention, named in the challenge brief:* "Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations."
- *suppression, stated in NASA's objective for the investigation:* "BASS-II results contribute to the combustion computational models used in the design of fire detection and suppression systems in microgravity and on Earth."
- *detection, stated in NASA's objective for the investigation:* "BASS-II results contribute to the combustion computational models used in the design of fire detection and suppression systems in microgravity and on Earth."

## BASS (PSI-26)

**Burning and Suppression of Solids**. Microgravity Science Glovebox. Family: flow duct in the MSG. DOI 10.60555/58h4-dy80. Licence CC0-1.0.

**Objective (NASA).** The two main objectives for BASS are to: 1.) Begin to bridge the gap between the normal gravity NASA-STD-6001 Test#1 method, ground-based microgravity tests, and actual material flammability in microgravity. The relation between these can be investigated by observing flames burning for the longer times available in flight. Practical, realistic (thicker) fuels in typical geometries will be examined, including wake flames which have characteristics of a fire burning behind another object (for example, a fire burning in tight area which may not be easily accessible). 2.) Assess the effectiveness of an inert, gaseous extinguishing agent (similar to that used on ISS) in putting out flames over different materials, geometries, preheating, and flow.

**Approach (NASA).** The experiment was conducted aboard the International Space Station (ISS) inside the Microgravity Glove Box (MSG). Custom-made fabric samples consisting of 75% cotton and 25% fiberglass (by mass) measuring were used as a fuel source as well as Nomex fabric and Ultem. A Kanthal wire (29-gage)igniter was used to initiate combustion at one end of the fabric. Airflow was controlled using a variable speed fan, with airflow speeds reaching as high as 40 cm/s. The airflow was directed along the fuel surface in either the opposed or concurrent direction. This investigation explored two primary air flow configurations; concurrent-flow and opposed-flow. With concurrent-flow air flows in the same direction as the flame spread and in opposed-flow air flows in the opposite direction of flame spread. Flames were observed in the top and edge profiles and data was collected on flame size, shape, color, spread rate, and potential flame quenching. The samples were placed in a small flow duct (76 mm x 76 mm cross section) that allowed for airflow across the fabric. Other sample shapes were utilized as well including spheres and cylinders.

**Impacts, on Earth and in space (NASA).** Current NASA spacecraft materials selection is based on a standard test method (NASA–STD–6001 Test 1) that segregates material based on 1-g behavior without consideration of low gravity effects. A critical element of this understanding is the radiative heat emission from the flame. These results are used in first order models and predictions of heat release in spacecraft fires and as a means to extend heat release data from tests like the NASA cone calorimeter test (NASA–STD–6001 Test 2) to a performance-based material selection process. Using nitrogen as a flame suppressant in microgravity provides a direct link to current and planned extinguishment techniques. BASS results provide essential guidance to ground-based microgravity combustion research efforts. Detailed combustion models are validated using the simpler flow environment afforded by tests in microgravity. Once validated, they can be used to build more complex combustion models needed to capture the important details of flames burning in normal gravity. These models have wide applicability to the general understanding of many terrestrial combustion problems and will guide novel spacecraft design and ensure the safety of future crewed missions.

| | |
|---|---|
| Fuels or materials (rows) | not in the catalogue |
| Phase and configuration | unknown; unknown |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | stated by NASA, not per test |
| What varied between tests | not in the catalogue |
| Catalogue rows | no row (no row: this investigation is not in the test catalogue yet) |
| Files at NASA | 212 files, 163.6 GB; test ids in file names: 0 |
| Measurements per test | sensor series 0, images 35, video 12, processed data 0 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief); suppression (stated in NASA's objective for the investigation) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | used as outside data: ten burns were measured from the public stills and used once, for a candidate that was not adopted |
| Closest investigations | SLICE (same hardware family, same need: prevention); BASS-II (same hardware family, same need: prevention, suppression); SPICE (same hardware family) |

**How the tests differ.** Tests in the glovebox flow duct varied the material (cotton-fiberglass SIBAL fabric, Nomex, Ultem, PMMA spheres, thick flat PMMA and wax, wax candles), the geometry (1 and 2 cm wide strips, 1 and 2 cm spheres, thick slabs, candles, wake flames behind inert hemispheres), the flow direction (opposed or concurrent) and the air speed from 0 to 40 cm/s, the main variable. Some flames were vitiated or extinguished with nitrogen; later tests lowered oxygen below 21%. Normal-gravity tests served as comparisons.

**Instruments.** Nikon digital still camera through the top window (about 1 frame per second) (flame shape, flame length and flame spread over the sample); Colour video camera (30 frames per second, front view) (flame appearance, growth and extinction, with an overlay of fan setting, flow speed, radiometer signal and time); Hot-wire anemometer in the flow tunnel (air flow speed in the duct); Radiometer (flame radiant emission, recorded as a signal overlaid on the video); Cabin atmosphere readings logged per run (cabin oxygen, pressure and CO2 at the time of each burn).

**Findings, from NASA's documents.**

- In opposed flow the fabric flame quickly reached a steady spread rate at each flow speed, and spread was fastest at an intermediate flow speed rather than at the highest or lowest. *(NASA states; PSI-26_Presentations_BASS_Overview of Recent ISS Results.pdf, p. 26)*
- In concurrent flow a limiting flame length and steady spread rate were reached only at low flow speeds, where spread rate rose linearly with flow. *(NASA states; PSI-26_Presentations_Flammability Aspects of Fabric_USNationalCombustion2013.pdf, p. 36)*
- Reducing concurrent flow to around 1 cm/s produced quenching extinction, verifying the low-speed quenching branch of the U-shaped flammability boundary for this fabric. *(NASA states; PSI-26_Science Documents_FlammabilityAspectsofFabric_USNationalCombustion.2013.docx, p. 11)*
- The fabric showed enhanced flammability in microgravity: with a forced flow it kept spreading against the flow, whereas in normal-gravity air a flame spreading downward (opposed) self-extinguishes. *(NASA states; PSI-26_Presentations_Flammability Aspects of Fabric_USNationalCombustion2013.pdf, p. 35)*
- Ultem and Nomex HT90-40 did not burn in the ISS atmosphere (21 to 22% oxygen, flow around 15 cm/s), as in normal gravity, though the ISS samples were narrower; Nomex-III did burn. *(NASA states; PSI-26_Presentations_BASS_Overview of Recent ISS Results.pdf, p. 30)*
- The long-duration flights gave the first detailed transient flame-growth data in purely forced microgravity flow; they validate several theoretical predictions and feed a transient flame growth model. *(NASA states; PSI-26_Presentations_Flammability Aspects of Fabric_USNationalCombustion2013.pdf, p. 38)*

**Why it matters for fire safety** (NASA states). Spacecraft materials are screened by NASA-STD-6001 Test 1, a normal-gravity test that ignores low-gravity effects. BASS found a fabric burning in ventilated microgravity where it self-extinguishes in normal gravity, so the test may need re-evaluation for safety margin. Its nitrogen suppressant tests link directly to current and planned spacecraft extinguishment techniques.

- *prevention, named in the challenge brief:* "Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations."
- *suppression, stated in NASA's objective for the investigation:* "Assess the effectiveness of an inert, gaseous extinguishing agent (similar to that used on ISS) in putting out flames over different materials, geometries, preheating, and flow."

## SAFFIRE-I (PSI-98)

**Spacecraft Fire Safety Experiment-I**. Saffire 1 enclosure in Cygnus. Family: large samples in the Cygnus vehicle. DOI 10.60555/0t15-1z43. Licence CC0-1.0.

**Objective (NASA).** The objectives of this investigation were to: (1) determine how rapidly a large scale fire grows in low-gravity and (2) investigate the low-g flammability limits compared to those obtained in NASA’s normal gravity material flammability screening test.

**Approach (NASA).** The Saffire-1 experiments was performed on the Orbital Science’s Cygnus resupply vehicle after it deberthed from the ISS. The sample material for Saffire-I is a composite fabric called Sibal consisting of cotton on a fiberglass substrate (75% cotton, 25% fiberglass by weight). It has been used in many space experiments because it burns like cotton but maintains structural integrity after the cotton is consumed. The sample was approximately 0.94 meters long and 0.4 meters wide, simulating a large piece of flammable fabric. The Saffire-1 flight unit had a sample card installed in the middle of a large flow duct. Air is drawn through the flow duct by fans located at the top of the duct with flow straighteners at the bottom of the experiment module. Saffire-I was conducted at an air flow velocity of 20 cm/s. Once the sample was ignited flame images were captured using two cameras. In addition, thermocouples provided flow, flame, and plume temperatures, two radiometers recorded flame radiation, solid state sensors were used to measure pressure, CO2 and O2 mole faction, and two anemometer tracked flow velocity. All data was compressed, and subsequently downlinked.

**Impacts, on Earth and in space (NASA).** The Saffire-I experiment provides a new way to study a realistic fire on an exploration vehicle, which has not been possible in the past because the risks are too high. Instruments on an unmanned cargo vessel measure oxygen, carbon dioxide, heat, pressure and flame growth, and two video cameras provide views of the flame. Results determine microgravity flammability limits for several spacecraft materials, help to validate NASA’s material selection criteria, and how microgravity and limited oxygen affect flame size. The investigation is crucial for the safety of current and future space missions. Studying fire in small, sealed environments such as the Cygnus cargo supply vehicle benefits fire safety and prevention efforts on Earth, including inside mines, airplanes, or submarines.

| | |
|---|---|
| Fuels or materials (rows) | sibal_fabric (2) |
| Phase and configuration | solid; flat_sample |
| Sizes | sample length 406 to 406 mm; sample thickness 3.7 to 3.7 mm; sample width 940 to 940 mm |
| Ambient oxygen | 0.21 to 0.21 (1 rows) |
| Pressure | not in the table |
| Imposed flow | 20 to 20 cm/s (2 rows) |
| What varied between tests | direction |
| Catalogue rows | 2 tests (every row of NASA's experiment table) |
| Files at NASA | 76 files, 12.3 GB; test ids in file names: 2 |
| Measurements per test | sensor series 2, images 2, video 0, processed data 2 |
| Crew atmospheres | earth iss: 0 inside, 1 oxygen only; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | none: no model; the same fabric as BASS-II at forty times the width |
| Closest investigations | SAFFIRE-II (same hardware family, same fuel or material: sibal_fabric, same need: prevention); SAFFIRE-III (same hardware family, same fuel or material: sibal_fabric, same need: prevention); BASS-II (same fuel or material: sibal_fabric, same need: prevention) |

**How the tests differ.** Saffire-I burned large SIBAL fabric (75% cotton, 25% fiberglass), about 0.94 m by 0.4 m, in a flow duct aboard Cygnus after it left the ISS, in air of about 21.5% oxygen moving at 20 cm/s. The experimental table lists two SIBAL burns of this size, S1 and S2, which differ in spread mode: concurrent or opposed flow. Material, flow speed and oxygen were not varied; the companion Saffire-III used 30 cm/s and Saffire-II burned nine smaller samples.

**Instruments.** Two video cameras (Flame images of fire growth over the sample); Thermocouples (Flow, flame and plume temperatures); Two radiometers (Flame radiation); Solid-state gas and pressure sensors (Pressure and O2 and CO2 mole fractions); Two anemometers (Flow velocity in the duct); Smoke wire (Flow visualization, recorded as pre-test and post-test flow image sets).

**Findings, from NASA's documents.**

- NASA's hypothesis is that fire behaves so differently in reduced gravity that designing spacecraft fire safety systems from terrestrial fires and fire standards carries inherent risk, which grows as missions travel farther from Earth. *(NASA states; PSI metadata record, field hypothesis)*
- Large-scale low-gravity fire data were missing before Saffire: earlier spacecraft fire experiments used samples of at most about 10 cm. *(NASA states; PSI-98_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 1)*
- The design lesson behind Saffire is to run large fires in an expendable cargo vehicle after it leaves the station, so realistic fire sizes can be studied without risk to a crew. *(NASA states; PSI metadata record, field hypothesis)*
- NASA credits the Saffire results with determining microgravity flammability limits for several spacecraft materials, helping validate its material selection criteria, and showing how microgravity and limited oxygen affect flame size. *(NASA states; PSI metadata record, field research_impacts)*
- The Saffire team notes that the small number of tests leaves knowledge gaps and that detection, suppression and post-fire cleanup still need demonstration, mapping these objectives to Saffire-IV to VI and ground work. *(NASA states; PSI-98_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 5)*
- With one fabric, one flow speed and one oxygen level, Saffire-I gives a large-scale fire growth reference for a single condition, not a trend; the material-limit comparison NASA describes rests on Saffire-II. *(interpretation; PSI-98_Experimental table_SAFFIRE-1.csv, p. 1)*

**Why it matters for fire safety** (NASA states). NASA states Saffire-I is crucial for current and future mission safety: it measures how fast a large fabric fire grows in low gravity on a real vehicle, without crew risk, and supports checking NASA's normal-gravity material flammability screening and selection criteria against low-gravity behaviour.

- *prevention, named in the challenge brief:* "Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations."

## SAFFIRE-II (PSI-99)

**Spacecraft Fire Experiment-II**. Cygnus Pressurized Cargo Module, SAFFIRE. Family: large samples in the Cygnus vehicle. DOI 10.60555/2chp-m469. Licence CC0-1.0.

**Objective (NASA).** The objectives of SAFFIRE-II are to determine how rapidly a large scale fire grows in low-gravity and to investigate the low-g flammability limits compared to those obtained in NASA’s normal gravity material flammability screening test.

**Approach (NASA).** The Saffire-II experiment was performed on the Orbital Science’s Cygnus resupply vehicle after it deberthed from the ISS. As a follow on to the Saffire-I investigation, the Saffire-II investigation added more samples including SIBAL, silicone, structure PMMA, flat PMMA, and PMMA with Nomex. The Saffire-1I flight unit had a sample card installed in the middle of a large flow duct. Air is drawn through the flow duct by fans located at the top of the duct with flow straighteners at the bottom of the experiment module. Saffire-II has a sample card that contains 9 samples, each 5 cm x 29 cm. Depending on the sample material, the Saffire-II was conducted at an air flow velocity of either 20 or 30 cm/s, the flow being either concurrent or opposed. Once the sample was ignited flame images were captured using two cameras. In addition, thermocouples provided flow, flame, and plume temperatures, two radiometers recorded flame radiation, solid state sensors were used to measure pressure, CO2 and O2 mole faction, and two anemometer tracked flow velocity. All data was compressed, and subsequently downlinked. Details related to samples and experimental parameters are provided in the Experimental Details file in the Science document folder.

**Impacts, on Earth and in space (NASA).** The Saffire-II experiment provides a new way to study a realistic fire on an exploration vehicle, which has not been possible in the past because the risks are too high. Instruments on an unmanned cargo vessel measure oxygen, carbon dioxide, heat, pressure and flame growth, and two video cameras provide views of the flame. Results determine microgravity flammability limits for several spacecraft materials, help to validate NASA’s material selection criteria, and how microgravity and limited oxygen affect flame size. The investigation is crucial for the safety of current and future space missions. Studying fire in small, sealed environments such as the Cygnus cargo supply vehicle benefits fire safety and prevention efforts on Earth, including inside mines, airplanes, or submarines.

| | |
|---|---|
| Fuels or materials (rows) | silicone (4), sibal_fabric (2), pmma (2), pmma+nomex (1) |
| Phase and configuration | solid; flat_sample |
| Sizes | sample length 290 to 290 mm; sample thickness 0.27 to 10 mm; sample width 50 to 50 mm |
| Ambient oxygen | 0.21 to 0.22 (7 rows) |
| Pressure | not in the table |
| Imposed flow | 20 to 20 cm/s (9 rows) |
| What varied between tests | x o2, igniter power, igniter time, direction, name, sample length, sample thickness |
| Catalogue rows | 9 tests (every row of NASA's experiment table) |
| Files at NASA | 214 files, 16.8 GB; test ids in file names: 9 |
| Measurements per test | sensor series 9, images 9, video 0, processed data 9 |
| Crew atmospheres | earth iss: 0 inside, 1 oxygen only; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | none: no model |
| Closest investigations | SAFFIRE-I (same hardware family, same fuel or material: sibal_fabric, same need: prevention); SAFFIRE-III (same hardware family, same fuel or material: sibal_fabric, same need: prevention); BASS-II (same fuel or material: sibal_fabric, same need: prevention) |

**How the tests differ.** Saffire-II burned nine samples, each 5 cm by 29 cm, on one card in a flow duct aboard Cygnus after it left the ISS. Samples differed in material (silicone, SIBAL cotton-fiberglass, structure PMMA, flat PMMA, PMMA with Nomex) and thickness, in igniter position, and in flow direction (concurrent or opposed). NASA's record gives 20 or 30 cm/s air flow by material; the experimental table lists 20 cm/s and about 22% oxygen. The table also lists 1-g burn lengths and spread rates for comparison.

**Instruments.** Two video cameras (Flame images of each sample burn); Thermocouples (Flow, flame and plume temperatures); Two radiometers (Flame radiation); Solid-state gas and pressure sensors (Pressure and O2 and CO2 mole fractions); Two anemometers (Flow velocity in the duct); Smoke wire (Flow visualization, recorded as pre-flow and post-flow image sets).

**Findings, from NASA's documents.**

- NASA's hypothesis is that fire behaves so differently in reduced gravity that designing spacecraft fire safety systems from terrestrial fires and fire standards carries inherent risk, which grows as missions travel farther from Earth. *(NASA states; PSI metadata record, field hypothesis)*
- Saffire-II is the flammability-limit member of the Saffire series: its nine-material card is meant to establish maximum oxygen concentration (MOC) limits in low gravity for comparison with normal-gravity screening. *(NASA states; PSI-99_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 3)*
- Large-scale low-gravity fire data were missing before Saffire: earlier spacecraft fire experiments used samples of at most about 10 cm. *(NASA states; PSI-99_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 1)*
- The design lesson behind Saffire is to run fires in an expendable cargo vehicle after it leaves the station, so realistic fires can be studied without risk to a crew. *(NASA states; PSI metadata record, field hypothesis)*
- The Saffire team notes that the small number of tests leaves knowledge gaps and that detection, suppression and post-fire cleanup still need demonstration, mapping these objectives to Saffire-IV to VI and ground work. *(NASA states; PSI-99_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 5)*
- Because the experimental table gives about 22% oxygen for the burns, Saffire-II shows whether and how each material burns in low gravity near air oxygen, against its 1-g behaviour; it does not by itself locate an oxygen limit. *(interpretation; PSI-99_Experimental table_SAFFIRE-2.csv, p. 1)*

**Why it matters for fire safety** (NASA states). NASA states Saffire-II is crucial for current and future mission safety: by burning several spacecraft materials in low gravity on a real vehicle, it addresses microgravity flammability limits and the validity of NASA's normal-gravity material screening and selection criteria, and how microgravity and limited oxygen affect flame size.

- *prevention, named in the challenge brief:* "Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations."

## SAFFIRE-III (PSI-100)

**Spacecraft Fire Experiment-III**. Saffire III enclosure in Cygnus. Family: large samples in the Cygnus vehicle. DOI 10.60555/040q-4h36. Licence CC0-1.0.

**Objective (NASA).** The objectives of this investigation were to: (1) determine how rapidly a large scale fire grows in low-gravity and (2) investigate the low-g flammability limits compared to those obtained in NASA’s normal gravity material flammability screening test.

**Approach (NASA).** The Saffire-III experiments were performed on the Orbital Science’s Cygnus resupply vehicle after it deberthed from the ISS. As a follow on to the Saffire 1 investigation, the Saffire III investigation also used SIBAL as the sample material. However, the experiments were conducted at a flow velocity of 30 cm/s compared to 20 cm/s during Saffire I. The flow was either concurrent or opposed. Once the sample was ignited flame images were captured using two cameras. In addition, thermocouples provided flow, flame, and plume temperatures, two radiometers recorded flame radiation, solid state sensors were used to measure pressure, CO2 and O2 mole faction, and two anemometer tracked flow velocity. All data was compressed, and subsequently downlinked. Details related to samples and experimental parameters are provided in the Experimental Details file in the Science document folder.

**Impacts, on Earth and in space (NASA).** The Saffire-I experiment provides a new way to study a realistic fire on an exploration vehicle, which has not been possible in the past because the risks are too high. Instruments on an unmanned cargo vessel measure oxygen, carbon dioxide, heat, pressure and flame growth, and two video cameras provide views of the flame. Results determine microgravity flammability limits for several spacecraft materials, help to validate NASA’s material selection criteria, and how microgravity and limited oxygen affect flame size. The investigation is crucial for the safety of current and future space missions. Studying fire in small, sealed environments such as the Cygnus cargo supply vehicle benefits fire safety and prevention efforts on Earth, including inside mines, airplanes, or submarines.

| | |
|---|---|
| Fuels or materials (rows) | sibal_fabric (2) |
| Phase and configuration | solid; flat_sample |
| Sizes | sample length 940 to 940 mm; sample thickness 3.3 to 3.3 mm; sample width 406 to 406 mm |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | 30 to 30 cm/s (2 rows) |
| What varied between tests | direction |
| Catalogue rows | 2 tests (every row of NASA's experiment table) |
| Files at NASA | 87 files, 35.2 GB; test ids in file names: 2 |
| Measurements per test | sensor series 2, images 2, video 0, processed data 2 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | prevention (named in the challenge brief) |
| Towards partial gravity | normal-gravity comparison |
| Hand-written models | none: no model |
| Closest investigations | SAFFIRE-I (same hardware family, same fuel or material: sibal_fabric, same need: prevention); SAFFIRE-II (same hardware family, same fuel or material: sibal_fabric, same need: prevention); BASS-II (same fuel or material: sibal_fabric, same need: prevention) |

**How the tests differ.** Saffire-III burned two SIBAL fabric samples of the same size (0.33 cm thick, 94 cm long, 40.6 cm wide) in a flow duct aboard the uncrewed Cygnus vehicle after it left the ISS. The difference was flow direction: sample S1 burned in concurrent flow and S2 in opposed flow, both in 30 cm/s air, against 20 cm/s in Saffire-I. No change of material or atmosphere between the samples is listed.

**Instruments.** Two cameras (Flame images); Thermocouples (Flow, flame and plume temperatures); Two radiometers (Flame radiation); Solid state gas and pressure sensors (Pressure and CO2 and O2 mole fractions); Two anemometers (Flow velocity); Smoke wire (Flow visualization (listed in the diagnostics table)).

**Findings, from NASA's documents.**

- Before Saffire, spacecraft fire experiments had used samples of at most 10 cm, so large-scale data on fire growth in low gravity were missing; this gap motivates the Saffire series. *(NASA states; PSI-100_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 1)*
- NASA's hypothesis is that designing spacecraft fire-safety systems from terrestrial fires and fire standards carries inherent risk, because fire behaves very differently in reduced gravity, and that this risk grows on missions farther from Earth. *(NASA states; PSI metadata record, field hypothesis)*
- Running the burns in an expendable cargo vehicle after it leaves the ISS makes large-scale fire experiments possible without risk to a crew. *(NASA states; PSI metadata record, field hypothesis)*
- Within the series, Saffire-I and -III burn a large SIBAL fabric sample to study low-gravity flame spread, while Saffire-II carries a nine-material sample card to study flammability limits. *(NASA states; PSI-100_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 2)*
- The 2016 paper maps spacecraft fire-safety objectives to Saffire-I to -III, the follow-on Saffire-IV to -VI and ground work, and notes the small number of tests and the need for later detection, suppression and cleanup demonstrations. *(NASA states; PSI-100_Science Documents_SAFFIRE Objectives, Development and Status.pdf, p. 5)*
- Saffire-III's design (one large sample per flow direction, no repeats) serves the first objective, large-fire growth, as a single comparison of concurrent and opposed spread; the flammability-limit comparison in the second objective is carried by Saffire-II's nine-material card. *(interpretation; PSI-100_Experimental table_SAFFIRE-3.csv, p. 1)*

**Why it matters for fire safety** (NASA states). NASA states that realistic fires on an exploration vehicle could not be studied before because the risk was too high, and that Saffire measurements (oxygen, CO2, heat, pressure, flame growth) help determine material flammability limits and validate NASA's material selection criteria; the impacts text NASA gives for this record is written for Saffire-I.

- *prevention, named in the challenge brief:* "Flammability studies focus on solid materials such as fabrics and plastics, as studied in the Burning and Suppression of Solids (BASS and BASS-II) and Spacecraft Fire Safety Demonstration (Saffire-I, -II, and -III) investigations."

## SAME (PSI-102)

**Smoke and Aerosol Measurement Experiment**. Comparitive Soot Diagnostic Hardware. Family: smoke and aerosol instruments. DOI 10.60555/gd4k-8h62. Licence CC0-1.0.

**Objective (NASA).** One of the major goals of this investigation was to quantify the sizes that the liquid smoke particulate achieves in low gravity and to relate the droplet size distribution to the smoke generation and smoke transport conditions in the experiment. A numerical code was to be developed to predict the smoke droplet growth as a function of the fuel pyrolysis rate, the thermodynamic properties of the pyrolysis vapor, and the flow environment.

**Approach (NASA).** The crew installed the near field hardware in the Glovebox, attached the Far Field Box to the Glovebox, and positioned the video cameras. The operator then ran the self diagnostic procedures on the two smoke detectors and activated the video cameras, turned on the igniter for a predefined period of time (15 to 60 seconds), and initiated the thermophoretic soot samplers to sample the smoke. The actual duration of each combustion event was typically 2 minutes. After flight, the particulate collected on the thermophoretic probes were analyzed using a Transmission Electron microscope (TEM) to determine primary and aggregate particle dimensions. The remains of the samples were weighed to determine their mass loss. The digital data was transcribed from the video record and analyzed to determine the response of the detectors. Once the flight mass loss rates were determined, 1-g operation conditions to produce the same loss were determined.The measurements were made using smoke generated by overheated spacecraft materials where variables such as the sample temperature, flow field, and particle aging time were more rigorously controlled.

**Impacts, on Earth and in space (NASA).** The SAME experiment will provide technology for an advanced fire detector for future spacecraft that will be used for long duration missions. SAME will provide quantitative data on the sensitivity of these detectors to reduced gravity smokes that will allow evaluation of the adequacy of these existing technologies using relevant data. The current Fire Prevention, Detection, and Suppression (FPDS) program plan allows for the re-evaluation of future sensor technology, to allow new technology and capability to be utilized. The results from SAME are needed to provide the reduced gravity baseline data against which future detection technology developments can be evaluated.The smoke detectors developed from the results of SAME can also be useful in other extreme environments on Earth, such as submarines or underwater laboratories. Accurate detection of smoke in these environments can save lives.

| | |
|---|---|
| Fuels or materials (rows) | teflon (4), kapton (1), silicone_rubber (1), lampwick (1), dibutyl_phthalate (1) |
| Phase and configuration | solid, liquid_other; heated_wire_sample |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | 5 to 10 cm/s (4 rows) |
| What varied between tests | planned aging duration, speed, name, phase |
| Catalogue rows | 8 matrix rows (every row of NASA's experiment table) |
| Files at NASA | 101 files, 0.7 GB; test ids in file names: 42 |
| Measurements per test | sensor series 0, images 0, video 0, processed data 42 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | detection (named in the challenge brief) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model |
| Closest investigations | SAME-R (same hardware family, same fuel or material: kapton, lampwick, teflon, same need: detection); DAFT / DAFT-2 (same hardware family, same need: detection) |

**How the tests differ.** SAME heated five materials below ignition in the ISS glovebox: Teflon, Kapton, silicone rubber, cellulose lamp wick and a dibutyl phthalate (DBP) reference. The plan varied air flow (5 or 10 cm/s), weight-loss rate (1 or 2 mg/min) and aging time (1 or 20 min); the flight used about 2 to 8 cm/s airflow and aging of 10 to 1800 s, with DBP points repeated. The metadata approach text matches the earlier CSD shuttle procedure (igniter 15 to 60 s, thermophoretic probes).

**Instruments.** TSI P-Trak condensation particle counter (modified for low gravity) (zeroth moment M0 of the size distribution: particle number concentration); Ionization chamber (first moment M1 of the size distribution); Two TSI DustTraks with 1 um and 10 um impactors (third moment M3 of the size distribution: mass concentration below the two size cutoffs); ISS and Shuttle smoke detectors in the diagnostics duct (response of the two existing spacecraft smoke detectors to the test smokes); Thermal precipitator with TEM grids (smoke particles captured for post-flight electron microscopy of size and morphology); Video camera and sample temperature recording (visual monitoring of the heated sample and its temperature).

**Findings, from NASA's documents.**

- A substantial portion of the smoke particles from the overheated spacecraft materials was smaller than 500 nm. *(NASA states; PSI-102_Reports_SAME_Final Science Report.pdf, p. 4)*
- The AIAA 2008 paper included in the report concludes that low-gravity count mean diameters were about 50 percent larger, yet still below 300 nm. *(NASA states; PSI-102_Reports_SAME_Final Science Report.pdf, p. 66)*
- Holding smoke in the aging chamber let particles coagulate: for spacecraft materials number counts fell by a factor of 1.7 to 2.7 over 800 s, count mean diameters grew about 1.5 times, and coagulation coefficients were repeatable. *(NASA states; PSI-102_Presentations_MeasurementofSmokeParticleSizeUnderLowGravityConditions.ppt, p. 32)*
- Size distributions depended on the material: Kapton and Teflon smokes had narrow distributions, while silicone rubber and lamp wick smokes had broad ones. *(NASA states; PSI-102_Reports_SAME_Final Science Report.pdf, p. 28)*
- Zero-gravity smoke transport modelling of the US Lab module showed that mixing dominates, so smoke detection lags are longer than in normal gravity. *(NASA states; PSI-102_Reports_SAME_Final Science Report.pdf, p. 4)*
- Combining SAME with the earlier CSD results, the team concludes that detecting smoke over this size range while rejecting dust likely needs a detector that measures several moments or classifies particle size. *(NASA states; PSI-102_Reports_SAME_Final Science Report.pdf, p. 29)*

**Why it matters for fire safety** (NASA states). SAME supplies the reduced-gravity smoke-size baseline NASA says is needed to evaluate current and future spacecraft smoke detectors. Its results argue for multi-moment or size-selective detection to catch sub-micron smoke while rejecting dust, and its Destiny modelling shows obstructions and stowage change detection times.

- *detection, named in the challenge brief:* "Fire detection was tested in microgravity through the Smoke and Aerosol Measurement Experiment studies (SAME and SAME-R)."

## SAME-R (PSI-101)

**Smoke Aerosol Measurement Experiment-Reflight**. Comparitive Soot Diagnostic Hardware; Refer to final report. Family: smoke and aerosol instruments. DOI 10.60555/2np7-jv14. Licence CC0-1.0.

**Objective (NASA).** Similar to the original SAME investigation, the primary objective of the SAME-R investigation was to quantify the sizes that the liquid smoke particulate achieves in low gravity and to relate the droplet size distribution to the smoke generation and smoke transport conditions in the experiment. One of the problems encountered during the SAME investigation was that after the thermal precipitators were returned to the ground, it was found that all the TEM grids were severely contaminated with some unidentified foreign particles, making these grids unusable although some of them showed shapes of certain types of smoke particles. Based on that experience the SAME-R investigation employed several modification covering new materials, extended operating conditions, and diagnostic equipments.

**Approach (NASA).** SAME-R utilized the SAME hardware used on-orbit. Based on the experience gained from SAME, SAME-R included the following modifications: 1) a new test material, Pyrell®, an intumescent fire-retardant foam by Foamex, was introduced to replace DBP samples, 2) test conditions were broadened including higher heating temperatures and no-flow during sample heating time, and 3) a new instrument, M-pass, developed in-house GRC, was used to replace one of the two Commercial Diagnostic Enclosures that includes a DustTrak and an ionization detector used in SAME. SAME-R flight tests were conducted from July 16, to September 9, 2010, successfully completing 66 test points. After SAME-R flight tests, three major post–flight ground tests were performed to improve data reduction process and to validate the lognormal distribution assumption, the key to the moment method employed by SAME and SAME-R.

**Impacts, on Earth and in space (NASA).** Along with the results from the original SAME investigation, the results from SAME-R can lead to technology for an advanced fire detector for future spacecraft that will be used for long duration missions. SAME will provide quantitative data on the sensitivity of these detectors to reduced gravity smokes that will allow evaluation of the adequacy of these existing technologies using relevant data. The current Fire Prevention, Detection, and Suppression (FPDS) program plan allows for the re-evaluation of future sensor technology, to allow new technology and capability to be utilized. The results from SAME are needed to provide the reduced gravity baseline data against which future detection technology developments can be evaluated. The smoke detectors developed from the results of SAME can also be useful in other extreme environments on Earth, such as submarines or underwater laboratories. Accurate detection of smoke in these environments can save lives.

| | |
|---|---|
| Fuels or materials (rows) | kapton (29), silicone (22), lampwick (20), pyrell (20), teflon (18) |
| Phase and configuration | solid; heated_wire_sample |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | 0 to 8 cm/s (109 rows) |
| What varied between tests | heater resistance ratio, heating duration, primary aging time, direction, speed, name |
| Catalogue rows | 67 tests, 42 test points (every row of NASA's experiment table) |
| Files at NASA | 344 files, 6.3 GB; test ids in file names: 67 |
| Measurements per test | sensor series 44, images 45, video 0, processed data 66 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | detection (named in the challenge brief) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model |
| Closest investigations | SAME (same hardware family, same fuel or material: kapton, lampwick, teflon, same need: detection); DAFT / DAFT-2 (same hardware family, same need: detection); SAFFIRE-II (same fuel or material: silicone) |

**How the tests differ.** Smoke came from sub-ignition heating of five spacecraft materials: Kapton, Teflon, silicone rubber, lamp wick and Pyrell foam (Pyrell replaced the DBP reference used in SAME). Across runs NASA varied heating temperature (wire resistance ratio Rt/R0 about 1.22 to 1.48; baseline, mid and high), inlet air flow (8.1 cm/s, 2 cm/s low flow, and zero flow during heating) and smoke aging time (net about 21 to 1923 s). All flight runs were in microgravity in the ISS glovebox; post-flight ground tests checked the lognormal size assumption.

**Instruments.** TSI P-Trak condensation particle counter (zeroth moment of the smoke size distribution: particle number concentration); Ionization chamber (first moment of the smoke size distribution (diameter-weighted concentration)); TSI DustTrak aerosol monitor (third moment of the smoke size distribution: mass concentration); MPASS two-angle light-scattering sensor (new in SAME-R) (two approximate moments of the size distribution from light scattered at two angles); Thermal precipitator with TEM grids (smoke particles deposited for post-flight transmission electron microscopy of morphology and size); Shuttle (STS) dual-chamber ionization detector and ISS two-pass infrared forward-scattering detector (response of the two existing spacecraft smoke detectors to the test smokes); Sample heater wire resistance (sample heating temperature, derived from total wire resistance with a heater model); SMPS (post-flight ground drum tests) (size distributions of SAME smoke in aged and unaged drums, used to test the lognormal assumption).

**Findings, from NASA's documents.**

- Smoke size distributions differed among the five spacecraft materials, but every material produced sub-micron particles that an ionization detector can sense. *(NASA states; PSI-101_Reports_SAME-R_Final Report.pdf, p. 84)*
- Smoke particle sizes were log-normally distributed, mostly 100 to 600 nm, and grew substantially with longer aging and with lower air flow past the heated sample. *(NASA states; PSI-101_Presentations_SAME-2_Comparison of Flight and Ground Test Results.pptx, p. 27)*
- With no airflow during heating, smoke stayed confined to about 64 cm3 at much higher concentration and grew larger particles; NASA reports a 30 to 40 percent larger diameter of average mass across 11 zero-flow tests, with half the smoke lost through the inlet. *(NASA states; PSI-101_Reports_SAME-R_Final Report.pdf, p. 83)*
- Smoke particles grow by nucleation, condensation and coagulation; liquid smokes (lamp wick, silicone, Kapton) coalesce while solid smokes (Teflon, Pyrell) form fractal agglomerates, and aging mainly removes the smallest particles. *(NASA states; PSI-101_Reports_SAME-R_Final Report.pdf, p. 8)*
- Ground tests at 8 cm/s airflow gave particle sizes very close to the flight results; the ISS scattering detector performed worse for Kapton and Teflon smoke, and the Shuttle ionization detector slightly better. *(NASA states; PSI-101_Presentations_SAME-2_Comparison of Flight and Ground Test Results.pptx, p. 28)*
- A smoke detector that senses one moment of the size distribution is unlikely to be sensitive across all smoke sizes while resisting false alarms; detectors that normalise concentration for particle size may solve this. *(NASA states; PSI-101_Presentations_SAME-2_Comparison of Flight and Ground Test Results.pptx, p. 29)*

**Why it matters for fire safety** (NASA states). SAME-R gives microgravity smoke-size data from overheated spacecraft materials for designing and judging spacecraft smoke detectors. NASA states the results are needed as the reduced-gravity baseline for future detection technology, and that size-selective or multi-moment detection could improve sensitivity and false-alarm rejection.

- *detection, named in the challenge brief:* "Fire detection was tested in microgravity through the Smoke and Aerosol Measurement Experiment studies (SAME and SAME-R)."

## DAFT / DAFT-2 (PSI-47)

**Dust and Aerosol Measurement Feasibility Test / Dust and Aerosol Measurement Feasibility Test-2**. TSI P-Trak. Family: smoke and aerosol instruments. DOI 10.60555/ztgj-d202. Licence CC0-1.0.

**Objective (NASA).** The focus of DAFT / DAFT-2 was to mitigate the risk for the Smoke Aerosol Measurement Experiment (SAME). SAME relied on a commercially available condensate nuclei counter, the TSI P-Trak, to quantify smoke aerosols in a microgravity environment, however there were concerns that th P-Trak would not be reliable in microgravity as it relies on gravity to recycle isopropyl alcohol used in the measurement process. DAFT / DAFT-2 tests that a modified P-Trak particulate counter would be accurate in microgravity. Summarized objectives include: 1) Demonstrate the Functionality of the Modified TSI P-Trak in Microgravity 2) Compare the TSI P-Trak Performance against the DustTrak control 3) Create and Analyze Aerosol Standards in Microgravity 4) Ensure the reliability of future aerosol experiments

**Approach (NASA).** The experimental approach consists of the following components: 1) Modification of Instrumentation: TSI P-Trak was modified for use in microgravity by redesigning the condenser tube to use capillary forces for recycling isopropyl alcohol and removing the mercury switch responsible for detecting instrument orientation. 2) Aerosol Generation: Arizona Red Dust (ARD) was chosen as an aerosol standard due to its ease of dispersion. The ARD was aerosolized with GN2 to create particle samples. The aerosol was contained within 15-liter Mylar bags. 3) On-Orbit Testing: The modified P-Trak gathered data from counting aerosol particles and these data were compared to the DustTrak CNC specifically designed to work in microgravity.

**Impacts, on Earth and in space (NASA).** Extended duration space flight missions with larger vessels will require fire detection systems with increased reliability and sensitivity. This research will impact the development of novel fire and smoke detection systems for the International Space Station and future spacecrafts. Additionally, the successful adaptation of the TSI P-Trak for microgravity can inform the design of instruments for use in other challenging Earth environments, such as submarines or underwater laboratories where accurate detection of smoke can save lives.

| | |
|---|---|
| Fuels or materials (rows) | arizona_road_dust (2) |
| Phase and configuration | unknown; aerosol_bag |
| Ambient oxygen | not in the table |
| Pressure | not in the table |
| Imposed flow | none recorded |
| What varied between tests | sampled location |
| Catalogue rows | 2 test days (every row of NASA's experiment table) |
| Files at NASA | 33 files, 0.0 GB; test ids in file names: 6 |
| Measurements per test | sensor series 6, images 0, video 0, processed data 6 |
| Crew atmospheres | earth iss: 0 inside; moon base rover: 0 inside; spacesuit: 0 inside |
| Fire-safety need | detection (stated in NASA's objective for the investigation) |
| Towards partial gravity | nothing |
| Hand-written models | none: no model |
| Closest investigations | SAME (same hardware family, same need: detection); SAME-R (same hardware family, same need: detection) |

**How the tests differ.** Sessions differed in what was sampled: D1 and D2 (2005) sampled undisturbed ISS cabin air at several locations (near EXPRESS racks, Destiny and Node 1) plus dust released by separating Velcro; D5 (2006) sampled three 15 L Mylar bags of Arizona Road Dust dispersed with nitrogen, using an impactor that removes particles above 2.5 micrometres. P-Trak counts were compared with DustTrak mass readings throughout; no material was burned.

**Instruments.** Modified TSI P-Trak condensation particle counter (particle number concentration, counted after isopropyl alcohol vapour condenses on the particles; capillary condenser for microgravity); TSI DustTrak aerosol monitor (hardened) (aerosol mass concentration by light scattering, gravity insensitive, used as the comparator); DAFT Impactor (removes particles above 2.5 micrometres from the test aerosol before sampling); HEPA filter zero check (instrument zero before and after each sampling period); Video (timing of HEPA filter and sample hose events, used to synchronise the instrument logs).

**Findings, from NASA's documents.**

- The commercial P-Trak relies on gravity to return condensed alcohol; using capillary forces in the condenser and removing the mercury orientation switch adapted it for microgravity. *(NASA states; PSI-47_Reports_DAFT_FinalReport.doc, p. 3)*
- The modified P-Trak worked reliably in low gravity, retiring the risk to SAME; SAME later flew in Increment 15 with flawless P-Trak operation. *(NASA states; PSI-47_Reports_DAFT_FinalReport.doc, p. 24)*
- Undisturbed ISS cabin air carries very little particulate, which the team attributes to HEPA filtration. *(NASA states; PSI-47_Presentations_DAFT_Increment13SciSymposium_2006.ppt, p. 10)*
- On orbit the particle count-to-mass ratio fell below the ground relation, which the team reads as larger particles staying airborne in microgravity. *(NASA states; PSI-47_Reports_DAFT_FinalReport.doc, p. 9)*
- Only three test points were run, limited by crew time, isopropyl alcohol supply and particle build-up in the P-Trak optics chamber. *(NASA states; PSI-47_Reports_DAFT_FinalReport.doc, p. 22)*
- A clean cabin background helps smoke stand out, but since larger particles stay airborne in orbit, dust raised by crew activity could challenge light-scattering smoke detectors; DAFT itself did not test any detector. *(interpretation; PSI-47_Reports_DAFT_FinalReport.doc, p. 9)*

**Why it matters for fire safety** (NASA states). NASA states that longer missions need more reliable and sensitive fire detection; DAFT supported this by qualifying the particle counter SAME used to size smoke for detector design, and by measuring the low particulate background of ISS cabin air.

- *detection, stated in NASA's objective for the investigation:* "The focus of DAFT / DAFT-2 was to mitigate the risk for the Smoke Aerosol Measurement Experiment (SAME)."
- *the brief also says:* "One of the instruments used for smoke detection in the SAME and SAME-R studies was instead used to detect dust in the Dust and Aerosol Measurement Feasibility Test (DAFT and DAFT-2) studies."

