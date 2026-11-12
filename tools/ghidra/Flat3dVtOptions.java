// Flat to 3D (GPL-3.0). Set the Auto Version Tracking options for the Spider-Man 2 address map run.
// A pre-script for tools/re/analyze.sh vt: it puts the options map where
// AutoVersionTrackingScript.java reads it (state environment variable "autoVTOptionsMap").
// Modelled on Ghidra's SetAutoVersionTrackingOptionsScript.java
// (Ghidra/Features/VersionTracking/ghidra_scripts).
//@category Version Tracking
import ghidra.app.script.GhidraScript;
import ghidra.feature.vt.gui.util.VTOptionDefines;
import ghidra.features.base.values.GhidraValuesMap;

public class Flat3dVtOptions extends GhidraScript {

	@Override
	protected void run() throws Exception {
		GhidraValuesMap map = new GhidraValuesMap();

		map.defineBoolean(VTOptionDefines.CREATE_IMPLIED_MATCHES_OPTION_TEXT, true);

		map.defineBoolean(VTOptionDefines.RUN_EXACT_SYMBOL_OPTION_TEXT, true);
		map.defineBoolean(VTOptionDefines.RUN_EXACT_DATA_OPTION_TEXT, true);
		map.defineBoolean(VTOptionDefines.RUN_EXACT_FUNCTION_BYTES_OPTION_TEXT, true);
		map.defineBoolean(VTOptionDefines.RUN_EXACT_FUNCTION_INST_OPTION_TEXT, true);
		map.defineBoolean(VTOptionDefines.RUN_DUPE_FUNCTION_OPTION_TEXT, true);
		map.defineBoolean(VTOptionDefines.RUN_REF_CORRELATORS_OPTION_TEXT, true);

		map.defineInt(VTOptionDefines.DATA_CORRELATOR_MIN_LEN_OPTION_TEXT, 5);
		map.defineInt(VTOptionDefines.SYMBOL_CORRELATOR_MIN_LEN_OPTION_TEXT, 3);
		map.defineInt(VTOptionDefines.FUNCTION_CORRELATOR_MIN_LEN_OPTION_TEXT, 10);
		map.defineInt(VTOptionDefines.DUPE_FUNCTION_CORRELATOR_MIN_LEN_OPTION_TEXT, 10);

		map.defineBoolean(VTOptionDefines.APPLY_IMPLIED_MATCHES_OPTION_TEXT, true);
		map.defineInt(VTOptionDefines.MIN_VOTES_OPTION_TEXT, 2);
		map.defineInt(VTOptionDefines.MAX_CONFLICTS_OPTION_TEXT, 0);

		map.defineDouble(VTOptionDefines.REF_CORRELATOR_MIN_SCORE_OPTION_TEXT, 0.95d);
		map.defineDouble(VTOptionDefines.REF_CORRELATOR_MIN_CONF_OPTION_TEXT, 10.0d);

		state.addEnvironmentVar("autoVTOptionsMap", map);
	}
}
