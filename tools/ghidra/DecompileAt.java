// Flat to 3D (GPL-3.0). Offline, read-only: decompiles functions at the given RVAs of an imported exe.
// usage: analyzeHeadless <proj> <name> -import EXE -noanalysis -scriptPath tools/ghidra -postScript DecompileAt.java OUT.c RVA...
// No auto-analysis is needed; each RVA is disassembled and made a function first. Writes only OUT.c.
import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;
import ghidra.app.cmd.disassemble.DisassembleCommand;
import ghidra.app.cmd.function.CreateFunctionCmd;
import java.io.*;

public class DecompileAt extends GhidraScript {
    public void run() throws Exception {
        String[] a = getScriptArgs();
        String out = a[0];
        long base = currentProgram.getImageBase().getOffset();
        DecompInterface d = new DecompInterface();
        d.openProgram(currentProgram);
        try (PrintWriter w = new PrintWriter(new FileWriter(out))) {
            for (int i = 1; i < a.length; i++) {
                long rva = Long.decode(a[i]);
                Address ad = toAddr(base + rva);
                new DisassembleCommand(ad, null, true).applyTo(currentProgram, monitor);
                Function f = getFunctionAt(ad);
                if (f == null) { new CreateFunctionCmd(ad).applyTo(currentProgram, monitor); f = getFunctionAt(ad); }
                w.println("==== " + a[i] + " " + (f == null ? "NO FUNCTION" : f.getName()));
                if (f == null) continue;
                DecompileResults r = d.decompileFunction(f, 120, monitor);
                w.println(r.decompileCompleted() ? r.getDecompiledFunction().getC() : ("FAILED " + r.getErrorMessage()));
            }
        }
    }
}
