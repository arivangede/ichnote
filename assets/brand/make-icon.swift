// Usage: make-icon <logo.png> <out.png>
// Crops the dark "I" mark (above the wordmark) and centers it on a macOS-style
// rounded square (824px inside a 1024px transparent canvas) in the logo's background color.
import AppKit
import CoreGraphics

let args = CommandLine.arguments
guard let src = NSImage(contentsOfFile: args[1]),
      let cg = src.cgImage(forProposedRect: nil, context: nil, hints: nil) else { fatalError("load") }
let w = cg.width, h = cg.height
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w * 4,
                    space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.draw(cg, in: CGRect(x: 0, y: 0, width: w, height: h))
let px = ctx.data!.bindMemory(to: UInt8.self, capacity: w * h * 4)
func lum(_ x: Int, _ y: Int) -> Int { let i = (y * w + x) * 4; return (Int(px[i]) + Int(px[i+1]) + Int(px[i+2])) / 3 }
// Buffer row 0 is the top of the image. Wordmark starts around y≈780; scan only above it.
var minX = w, maxX = 0, minY = h, maxY = 0
for y in 0..<760 { for x in 0..<w where lum(x, y) < 100 {
  minX = min(minX, x); maxX = max(maxX, x); minY = min(minY, y); maxY = max(maxY, y) } }
let bi = (10 * w + 10) * 4
let bg = (CGFloat(px[bi]) / 255, CGFloat(px[bi+1]) / 255, CGFloat(px[bi+2]) / 255)
print("mark bbox x:\(minX)-\(maxX) y:\(minY)-\(maxY) bg:\(bg)")
let crop = cg.cropping(to: CGRect(x: minX - 4, y: minY - 4, width: maxX - minX + 9, height: maxY - minY + 9))!

let S = 1024, inset: CGFloat = 100, side: CGFloat = 824
let out = CGContext(data: nil, width: S, height: S, bitsPerComponent: 8, bytesPerRow: S * 4,
                    space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
out.interpolationQuality = .high
let rect = CGRect(x: inset, y: inset, width: side, height: side)
out.addPath(CGPath(roundedRect: rect, cornerWidth: side * 0.2237, cornerHeight: side * 0.2237, transform: nil))
out.setFillColor(red: bg.0, green: bg.1, blue: bg.2, alpha: 1)
out.fillPath()
// Mark height = 62% of the rounded square.
let mh = side * 0.62, mw = mh * CGFloat(crop.width) / CGFloat(crop.height)
out.draw(crop, in: CGRect(x: (CGFloat(S) - mw) / 2, y: (CGFloat(S) - mh) / 2, width: mw, height: mh))
let rep = NSBitmapImageRep(cgImage: out.makeImage()!)
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: args[2]))
