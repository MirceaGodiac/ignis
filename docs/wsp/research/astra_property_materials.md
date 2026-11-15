# Shared material properties: source-bound continuation

Author: `gpt-6-astra`. Status: hand-authored, outcome-blind W0 property data; not a model release or a claim of experimental qualification. This continuation leaves `property_catalog.rs` and `catalog-w0-astra/` unchanged. No NASA report, experimental outcome, development result, holdout, old family forward or other author's numeric candidate was read. Material identities are joins; they select no physical law.

## Admitted content and its limit

The new catalog admits **one material/property series with nineteen paired points**: liquid n-heptane saturation-path molar heat capacity, `C_sat`, at ITS-90 temperatures 190, 200, 210, 220, 230, 240, 250, 260, 270, 280, 290, 298.15, 300, 310, 320, 330, 340, 350 and 360 K. The primary NBS and comparison Bureau of Mines columns were manually transcribed from printed p58, Table 2 of [Zabransky and Ruzicka (1994)](https://www.nist.gov/system/files/documents/srd/jpcrd469.pdf). Raw and SI units are both J mol^-1 K^-1; no conversion factor or fitted coefficient is introduced.

For example, at 298.15 K the columns are respectively **224.88 and 225.08 J mol^-1 K^-1**. The reader returns both and their signed arithmetic difference, with both dependency identities. It does not average them, tune a choice, treat their difference as uncertainty, or reduce uncertainty by counting shared sources twice. The NBS column was designated as primary prospectively because it is the first laboratory column, without using agreement or any combustion outcome to choose it.

These are **published, separately smoothed and reprocessed determinations**, not raw observed table points and not a fit performed here. The reanalysis converts both temperature and heat capacity to ITS-90. Its NBS column combines NBS measurements; it is not solely the Douglas 1954 experiment. The Bureau of Mines column combines its selected experiments. The reanalysis's final joint reference table is **not** used as a second source. Its extrapolated triple-point and upper BM rows are excluded.

Admission means that two distinct underlying experimental determinations, their original methods and their common comparison state have been checked. It does **not** mean statistically independent errors. Shared dependencies remain explicit: ITS-90 reprocessing, source molar-mass convention, auxiliary density/vapor-pressure descriptions, NBS calibration standards, and partially shared Calorimetry Conference samples. The underlying auxiliary originals have not been independently audited.

The thermodynamic quantity is `C_sat = T (dS/dT)_sat`, on the pure-liquid coexistence curve. It is neither ordinary constant-pressure `C_p` nor the derivative of saturated-liquid enthalpy. The pressure is the saturation pressure at each temperature; no atmospheric pressure is invented. No enthalpy or entropy additive zero is needed for this derivative. The source's legacy molar-mass convention is retained, rather than silently substituting a modern molar mass. Only the listed points are admitted: no interpolation, extrapolation, mixture law or conversion to mass-specific `C_p` is supplied.

Original evidence checked:

- [Douglas, Furukawa, McCoskey and Ball (1954)](https://nvlpubs.nist.gov/nistpubs/jres/53/jresv53n3p139_A1b.pdf), printed pp140-144: calorimeter, thermometer calibration, high/low filling experiments, raw table and the saturation-path correction. Their quoted 0.1% probable error above 50 K is explicitly an authors' non-statistical estimate, not a standard deviation.
- [Osborne and Ginnings (1947)](https://nvlpubs.nist.gov/nistpubs/jres/39/jresv39n5p453_A1b.pdf), printed pp475-477, especially Table 4: earlier distinct NBS calorimetry entering the later NBS composite. Source smoothing equations were visible but were not imported.
- [McCullough and Messerly, Bureau of Mines Bulletin 596 (1961)](https://digital.library.unt.edu/ark:/67531/metadc12750/m2/1/high_res_d/Bulletin0596.pdf), printed pp3-5 and 9-10: separate apparatus, vessels, thermometers, samples, selected heat capacities, and claimed 0.2% accuracy. This is a historical claimed accuracy, not a standard deviation or total uncertainty for the transformed points.

Total uncertainty and covariance of the transformed individual points remain explicitly **unquantified**. Historical experimental error statements are preserved with their original scope and meaning; they are not used as a numerical admission threshold.

## Pending content and correction of state semantics

Three new pending records retain the previously transcribed ideal-gas heat capacities at 298.15 K: methane 35.69, ethylene 42.90 and ethane 52.49 J mol^-1 K^-1. Their original independent numerical determinations and detailed temperature-scale/calculation lineage have not been verified, so the evaluator refuses them.

The [NIST WebBook guide](https://webbook.nist.gov/chemistry/guide/) explicitly identifies `C_p,gas` as **ideal-gas heat capacity**. The older frozen catalog's one-bar point representation should therefore not be used to interpret those pending rows as real finite-pressure measurements. The new pending records use `IdealGasLimit`; a standard-state pressure note is not a physical pressure-dependent measurement. The frozen records remain pending and have not been mutated.

The earlier liquid pairs for methanol, n-heptane and n-dodecane are still pending. The new n-heptane `C_sat` evidence does not promote the older ordinary-`C_p` pair. Methanol original-source retrieval was unsuccessful, and the necessary distinction between constant-pressure and saturation-path quantities is not assumed away. Dodecane's discrepant table values remain separate; no average is supplied.

All fifteen prospective component identities have explicit coverage gaps in the new dataset. Oxygen, nitrogen, helium, argon, carbon dioxide, water, propane, n-butane and farnesane have no admitted material property here. Methane, ethylene and ethane have only the pending points above; methanol and n-dodecane remain pending. Heptane has only the nineteen listed `C_sat` pairs. Density, actual isobaric heat capacity, latent heat, viscosity, conductivity, diffusivity, radiation and chemical-rate coverage remain missing. No mixture, commercial fuel formulation, solid or stereochemical composition has been invented. This is not a completeness claim for any investigation inventory.

## Reader and tests

`crates/ignis-wsp/src/property_materials.rs` is an internal typed reader, not a new cross-lane wire contract. Entry points are `MaterialCatalog::bundled`, `from_json`, `validate`, and `evaluate(series_id, &MaterialState, MaterialQuantity)`. Accessors expose the source, dependency, series and coverage inventories.

`MaterialState` carries temperature, temperature scale, thermodynamic path and composition identity. The evaluator validates even a catalog obtained by direct serde deserialization. It rejects pending records, wrong heat-capacity quantity, unresolved admitted states, wrong composition, a pressure/path mismatch, and any temperature absent from the table. There are no material-name branches. Returned values retain the series and point references, primary and comparison values, signed difference, deduplicated source identities and processing dependencies.

Validation checks finite positive data, raw decimal agreement, identity SI conversions, dimensional units, unique increasing temperature points, complete comparison columns, source pins, original experimental provenance and exact declarations of shared dependencies. A repeated experiment or overlapping original source set cannot be promoted as independent corroboration. A shared tabulation must appear in the shared dependency graph.

Five invariant tests cover bundled hash/roundtrip/dependency preservation; thermodynamic-domain and quantity refusal; identity-join behavior and pending ideal-gas refusal; false corroboration/missing dependencies/unpinned originals; and units/nonfinite/transcription/duplicate/schema rejection. The root agent reports `cargo test -p ignis-wsp property_materials --quiet` **passed all five tests, exit 0**, using its private `runs/tooling/astra-target`. This is the focused material-reader test result, not whole-workspace or scientific qualification. No Cargo build or test was launched by this author.

Executed locally: `rustfmt --edition 2021 --check crates/ignis-wsp/src/property_materials.rs` passed. `configs/models/properties/catalog-materials-astra/verify_sources.ps1` passed, checking the dataset hash and all eight declared source snapshots. The verifier performs only byte hashing and metadata checks; it never extracts, selects or generates physical values.

## Numeric origin inventory and source freeze

All physical numbers were manually typed as literals by `gpt-6-astra`. The nineteen temperatures and thirty-eight paired `C_sat` values have one exact table/column locator and distinct laboratory lineage per column. The three pending gas temperatures and heat capacities have WebBook row locators and remain non-admitted. The two historical accuracy percentages are source statements with separate locators and are not predictive weights. Test example values repeat source literals; deliberately invalid temperatures/pressure and alias values in negative tests are synthetic invariants, not observations. Structural integers in code are format/hash/index checks, not physical parameters.

The only derived material quantity computed here is `comparison - primary`, retaining both source dependencies. No optimizer, regression, parameter scan, coefficient-array import, synthetic replacement property or inference from combustion results was used. This source catalog is not itself a `HandwrittenWeightPatch` or a WSP model release; any later conversion into release weights must preserve the project's literal-patch and construction-manifest rules.

Dataset SHA-256: `1e2e38b0f2ae0582d5899eb0fff95c6816d981ffcb4010780bcc4ec7fe86ea06`. `materials.sha256` is checked by the bundled reader. `source-index.json` maps each declared source ID to its archived bytes; source hashes reside in the literal dataset and were verified against those bytes. Runtime lookup verifies the dataset pin; it does not reread the archived PDFs. The separate source verifier checks their hashes.

## Read extent and incidental exposure

The material dataset records precise read extents for its eight cited sources. The complete seven-page 1994 reanalysis was visually read. The 1954 PDF was text-extracted in full for searching, but only printed pp139-140 text and pp140-144 images were substantively read. The 1947 PDF was likewise extracted in full; only its search abstract and printed pp473,475-477 images were read. The scanned 1961 bulletin was visually read at PDF pages 1,3-9,13-14 (cover, contents, blank page and printed pp1-5,9-10); other pages were rendered but not inspected. Rendering and extraction did not populate the material dataset. The UNT landing record credits unrelated authors; the actual bulletin cover establishes McCullough and Messerly, and controls the citation.

Additional generic-property discovery reads, used only to locate possible originals and not to admit values: initial pages/selected search passages of Goodwin's 1987 methanol paper, DOI 10.1063/1.555786; selected table/method/reference passages of Setzmann and Wagner's 1991 methane paper, DOI 10.1063/1.555898; and search snippets for methane ideal-gas and compressed-liquid heat-capacity papers. Published correlation coefficients incidentally visible in these or the heptane originals were not imported or used. A methanol Carlson/Westrum PDF request returned HTML, not the paper; it is stored with an explicit `download-failed.html` name and is not a source snapshot supporting a property. The U Michigan landing attempt did not expose the original paper.

Broad generic-property searches incidentally returned snippets for a NASA generic methane bibliography and papers discussing NASA polynomial representations. None was opened or used; no microgravity-combustion outcomes were read. Other secondary-site and unrelated hydrocarbon snippets were ignored. The work used no documentary conditions processor, upload, measurement extraction pipeline, historical handoff, model result, goal/dossier or feedback bundle. The droplet author's later API-only message provided requested quantities and validity requirements without candidate numeric values or outcomes.

## Next consumer requirement

The frozen droplet kernel needs actual liquid density, ordinary mass-specific `C_p`, latent heat and saturation pressure at a consistent reference state, plus gas transport and heat capacities with justified domains. This catalog's `C_sat` cannot fill its liquid-`C_p` input. A room-temperature point also cannot justify a constant gas property across liquid, ambient and reaction-sheet temperatures. Further sources must establish those quantities and domains explicitly; gaps stay unavailable until then.

The droplet author confirmed that evaporation-only operation removes the combustion heat and reaction-sheet state, but still needs the other liquid/gas properties over evolving surface and ambient temperatures. It cannot replace an established burning interval or infer liquid temperature from a diameter observation. No complete source-admitted droplet packet exists from this catalog.

| Frozen droplet dependency | Available for methanol/heptane from this work |
|---|---|
| Liquid molar mass | No admitted paired property row; molecular identity alone is insufficient |
| Liquid density | Original candidate observations researched; pending state/source reconciliation |
| Actual isobaric liquid heat capacity | Missing; admitted heptane saturation-path heat capacity cannot substitute |
| Latent heat and saturation pressure at common reference | Missing admitted paired values |
| Complete-combustion heat for the established-burning branch | Missing |
| Representative gas density and common gas/vapor heat capacity | Missing applicable values and domain justification |
| Gas conductivity, diffusivity and viscosity | Missing applicable values and domain justification |
| Background and oxygen molar masses | No admitted paired property rows |
| Constant-property domains covering the evolving states | Unsupported; no domain extension was made |

## Follow-on liquid-density audit (non-admitted)

`catalog-materials-astra/droplet-liquid-pending.json` contains manually transcribed original heptane density observations and explicit conflicts. It is a research inventory, **not selectable by the material evaluator or droplet adapter**. Its SHA-256 is `adc381c072876527ca321972f675d4fa1511281b6059bd2c8b16588780262f2e`; all four new supporting snapshots were byte-hash verified. This audit does not change the frozen `materials.json` values or admission state.

- [Hevia et al., author manuscript arXiv:2409.05422](https://arxiv.org/pdf/2409.05422), Table 2 p10: the measured pure-heptane density is recorded at a stated nominal pressure. Methods identify a distinct Valladolid vibrating-tube experiment. Its density comparison against REFPROP is not a second independent measurement, and the temperature-scale/reference audit is not yet complete.
- [Cerdeirina et al. (2001), DOI 10.1039/B104891K](https://pubs.rsc.org/en/content/articlelanding/2001/cp/b104891k), original pp5230-5231: the Vigo density experiment is distinct, but the inspected text says atmospheric pressure without an exact number. More importantly for liquid `C_p`, its calorimeter was calibrated with n-heptane and toluene values from Zabransky; author-name differences cannot turn its heptane heat capacity into independent corroboration of that reference.
- [Pecar and Dolecek (2007)](https://acta-arhiv.chem-soc.si/54/54-3-538.pdf), Tables 1 and 2: different heptane densities are printed at the same nominal state. Both are retained, without choosing or averaging. Method and uncertainty statements do not erase the internal discrepancy.
- [Luning Prak, Cowart and Trulove (2014)](https://pubs.acs.org/doi/10.1021/je5007532), selected publisher-indexed methods and tables, cross-checked with its [NIST ThermoML archive](https://trc.nist.gov/ThermoML/10.1021/je5007532.html): the paper's pressure label and the archived pressure constraint differ, as do author and archive-compiler uncertainty assessments. The archive represents the same original experiment. These distinctions remain unresolved rather than being silently normalized.

The follow-on inventory gives raw/SI values, exact locators, phase, specimen notes, pressure statements, uncertainty meanings, source hashes and read extent per observation. It contains no fitted coefficients and performs no numerical selection. Reading the selected experimental ThermoML archive did not import its tables into executable inputs. Additional incidental search results included unrelated mixture-fit tables and kinetic-paper titles; no such model or combustion-outcome material was opened for this audit.
