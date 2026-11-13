// Flat to 3D (GPL-3.0). Write every match of a Version Tracking session to a CSV.
// A post-script for tools/re/analyze.sh vt, after AutoVersionTrackingScript.java:
//   -postScript ExportVtMatches.java /<session> <out.csv>
// args[0] is the session's project path, args[1] the CSV. Header:
//   src_rva,dst_rva,status,similarity,confidence,correlator
// Ghidra addresses are VAs, so each RVA is the address minus its own program's image base,
// written in hex with 0x. tools/re/fill_map.py --vt reads the file.
// It only reads the session: it never saves it (saveVersionTrackingSession throws).
//@category Version Tracking
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;

import ghidra.app.script.GhidraScript;
import ghidra.feature.vt.api.main.VTAssociation;
import ghidra.feature.vt.api.main.VTMatch;
import ghidra.feature.vt.api.main.VTMatchSet;
import ghidra.feature.vt.api.main.VTSession;
import ghidra.framework.model.DomainFile;
import ghidra.framework.model.DomainObject;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Program;

public class ExportVtMatches extends GhidraScript {

	@Override
	protected void run() throws Exception {
		String[] args = getScriptArgs();
		if (args.length != 2) {
			throw new IllegalArgumentException(
				"usage: ExportVtMatches.java <session path> <out.csv>; got " + args.length + " args");
		}
		DomainFile file = state.getProject().getProjectData().getFile(args[0]);
		if (file == null) {
			throw new IllegalArgumentException("ExportVtMatches: no session at " + args[0] +
				"; fix: pass the path AutoVersionTrackingScript.java wrote (/<session>)");
		}
		DomainObject obj = file.getDomainObject(this, true, true, monitor);
		try {
			if (!(obj instanceof VTSession)) {
				throw new IllegalArgumentException(
					"ExportVtMatches: " + args[0] + " is not a Version Tracking session");
			}
			VTSession session = (VTSession) obj;
			Program src = session.getSourceProgram();
			Program dst = session.getDestinationProgram();
			int rows = 0;
			try (PrintWriter out = new PrintWriter(args[1], StandardCharsets.UTF_8)) {
				out.println("src_rva,dst_rva,status,similarity,confidence,correlator");
				for (VTMatchSet set : session.getMatchSets()) {
					String correlator = cell(set.getProgramCorrelatorInfo().getName());
					for (VTMatch match : set.getMatches()) {
						VTAssociation a = match.getAssociation();
						out.println(rva(a.getSourceAddress(), src) + "," +
							rva(a.getDestinationAddress(), dst) + "," + a.getStatus() + "," +
							match.getSimilarityScore().getScore() + "," +
							match.getConfidenceScore().getScore() + "," + correlator);
						rows++;
					}
				}
			}
			println("ExportVtMatches: " + rows + " matches -> " + args[1]);
		}
		finally {
			obj.release(this);
		}
	}

	private static String rva(Address addr, Program program) {
		return "0x" + Long.toHexString(addr.subtract(program.getImageBase()));
	}

	private static String cell(String s) {
		if (s.indexOf(',') < 0 && s.indexOf('"') < 0 && s.indexOf('\n') < 0) {
			return s;
		}
		return "\"" + s.replace("\"", "\"\"") + "\"";
	}
}
