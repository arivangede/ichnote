// Usage: swift make-icon.swift <out.png>
// Draws the note-tice! mark (same geometry as notetice-mark.svg, measured from notetice-logo.png)
// in paper cream on a macOS-style near-black rounded square (824px inside a 1024px transparent
// canvas). Vector drawing keeps the icon crisp; the shadow in the source artwork is left out
// because macOS adds its own.
import AppKit
import CoreGraphics

let args = CommandLine.arguments
guard args.count == 2 else { fatalError("usage: make-icon <out.png>") }

// Mark geometry in logo coordinates (y down), viewBox 289 243 447 446.
let vb = CGRect(x: 289, y: 243, width: 447, height: 446)
let shapes: [[CGPoint]] = [
  [CGPoint(x: 289, y: 243), CGPoint(x: 398, y: 243), CGPoint(x: 398, y: 684), CGPoint(x: 289, y: 684)],
  [CGPoint(x: 422, y: 243), CGPoint(x: 615, y: 516), CGPoint(x: 615, y: 684), CGPoint(x: 422, y: 405)],
  [CGPoint(x: 629, y: 243), CGPoint(x: 736, y: 243), CGPoint(x: 682, y: 587)],
  [CGPoint(x: 682, y: 607), CGPoint(x: 723, y: 648), CGPoint(x: 682, y: 689), CGPoint(x: 641, y: 648)],
]

let S = 1024, inset: CGFloat = 100, side: CGFloat = 824
let out = CGContext(data: nil, width: S, height: S, bitsPerComponent: 8, bytesPerRow: S * 4,
                    space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
let rect = CGRect(x: inset, y: inset, width: side, height: side)
out.addPath(CGPath(roundedRect: rect, cornerWidth: side * 0.2237, cornerHeight: side * 0.2237, transform: nil))
out.setFillColor(red: 30 / 255, green: 30 / 255, blue: 30 / 255, alpha: 1)  // #1E1E1E
out.fillPath()

// Mark height = 56% of the rounded square, centred.
let scale = side * 0.56 / vb.height
let ox = (CGFloat(S) - vb.width * scale) / 2, oy = (CGFloat(S) - vb.height * scale) / 2
func map(_ p: CGPoint) -> CGPoint {  // logo coords (y down) -> context coords (y up)
  CGPoint(x: ox + (p.x - vb.minX) * scale, y: CGFloat(S) - (oy + (p.y - vb.minY) * scale))
}
out.setFillColor(red: 243 / 255, green: 241 / 255, blue: 229 / 255, alpha: 1)  // #F3F1E5
for poly in shapes {
  out.move(to: map(poly[0]))
  for p in poly.dropFirst() { out.addLine(to: map(p)) }
  out.closePath()
}
out.fillPath()

let rep = NSBitmapImageRep(cgImage: out.makeImage()!)
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[1]))
