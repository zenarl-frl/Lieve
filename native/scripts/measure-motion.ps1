# Requires a docked, unpinned Lieve in web spring mode; keep the pointer outside.
# Measures window-geometry event delivery, not compositor/GPU presentation FPS.
$ErrorActionPreference = 'Stop'
Add-Type @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
public class LieveMotionMeasure {
 [StructLayout(LayoutKind.Sequential)] struct Rect {public int L,T,R,B;}
 [StructLayout(LayoutKind.Sequential)] struct Point {public int X,Y;}
 [StructLayout(LayoutKind.Sequential)] struct Msg {public IntPtr H; public uint M; public UIntPtr W; public IntPtr L; public uint T; public Point P; public uint Private;}
 delegate void EventProc(IntPtr hook,uint e,IntPtr h,int obj,int child,uint thread,uint time);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string c,string t);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h,out uint p);
 [DllImport("user32.dll")] static extern IntPtr SetWinEventHook(uint min,uint max,IntPtr module,EventProc callback,uint pid,uint tid,uint flags);
 [DllImport("user32.dll")] static extern bool UnhookWinEvent(IntPtr hook);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h,out Rect r);
 [DllImport("user32.dll")] static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] static extern bool PeekMessage(out Msg m,IntPtr h,uint min,uint max,uint remove);
 [DllImport("user32.dll")] static extern bool TranslateMessage(ref Msg m);
 [DllImport("user32.dll")] static extern IntPtr DispatchMessage(ref Msg m);
 [DllImport("user32.dll")] static extern uint MsgWaitForMultipleObjectsEx(uint count,IntPtr handles,uint timeout,uint mask,uint flags);
 public static double[] Run() {
  var h=FindWindow("LieveNativeIsland",null); if(h==IntPtr.Zero) throw new Exception("Start Lieve first.");
  var dpi=SetThreadDpiAwarenessContext((IntPtr)(-4));
  bool pinned=false; IntPtr hook=IntPtr.Zero; EventProc callback=null;
  try {
   SendMessage(h,0x2A3,IntPtr.Zero,IntPtr.Zero); Pump(900);
   Rect r; GetWindowRect(h,out r);
   if((r.B-r.T)*96.0/GetDpiForWindow(h)>50) throw new Exception("Unpin and move the pointer outside first.");
   uint pid; GetWindowThreadProcessId(h,out pid);
   var deltas=new List<double>(); var watch=new Stopwatch(); double previous=-1;
   int width=-1,height=-1; bool recording=false;
   callback=delegate(IntPtr k,uint e,IntPtr w,int obj,int child,uint thread,uint time) {
    if(!recording || w!=h || obj!=0 || child!=0 || watch.Elapsed.TotalMilliseconds>300) return;
    Rect b; GetWindowRect(h,out b); int bw=b.R-b.L,bh=b.B-b.T;
    if(bw==width && bh==height) return; width=bw; height=bh;
    double at=watch.Elapsed.TotalMilliseconds;
    if(previous>=0) deltas.Add(at-previous); previous=at;
   };
   hook=SetWinEventHook(0x800B,0x800B,IntPtr.Zero,callback,pid,0,0);
   if(hook==IntPtr.Zero) throw new Exception("Cannot register geometry observer.");
   for(int i=0;i<4;i++) {
    previous=-1; width=-1; height=-1; watch.Restart(); recording=true;
    SendMessage(h,0x8001,IntPtr.Zero,(IntPtr)0x202); pinned=true; Pump(750); recording=false;
    SendMessage(h,0x8001,IntPtr.Zero,(IntPtr)0x202); pinned=false; Pump(750);
   }
   return deltas.ToArray();
  } finally {
   if(pinned) SendMessage(h,0x8001,IntPtr.Zero,(IntPtr)0x202);
   if(hook!=IntPtr.Zero) UnhookWinEvent(hook);
   GC.KeepAlive(callback); SetThreadDpiAwarenessContext(dpi);
  }
 }
 static void Pump(int milliseconds) {
  var watch=Stopwatch.StartNew(); Msg m;
  while(watch.ElapsedMilliseconds<milliseconds) {
   while(PeekMessage(out m,IntPtr.Zero,0,0,1)) {TranslateMessage(ref m); DispatchMessage(ref m);}
   MsgWaitForMultipleObjectsEx(0,IntPtr.Zero,5,0x04FF,4);
  }
 }
}
'@
$samples = @([LieveMotionMeasure]::Run() | Sort-Object)
if ($samples.Count -lt 8) { throw 'Too few animated geometry samples. Select Motion: web spring and retry.' }
[pscustomobject]@{
    Samples=$samples.Count
    MedianMilliseconds=[math]::Round($samples[[int][math]::Floor(($samples.Count-1)*0.5)],2)
    P95Milliseconds=[math]::Round($samples[[int][math]::Floor(($samples.Count-1)*0.95)],2)
    Over25ms=@($samples | Where-Object { $_ -gt 25 }).Count
}
