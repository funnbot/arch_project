```java
asc = new AsciiGridFileReader(new BufferedReader(new InputStreamReader(ascFile.getInputStream())));
GridDimensions dimensions = new GridDimensions(asc.getSize(), asc.getEnvelope());
DataBuffer data = asc.getData();
BandedSampleModel sampleModel = new BandedSampleModel(data.getDataType(), dimensions.getGridWidth(), dimensions.getGridHeight(), 1);
WritableRaster raster = Raster.createWritableRaster(sampleModel, data, null);
// no projections are used
// if (srcProj == null || dstProj == null || srcProj.equals(dstProj)) {
// return new RasterDataset(dimensions, raster);
// }

var9 = new RasterDataset(raster, dimensions, srcProj, dstProj);
```

```java
RasterDataset dataset = RasterDataset.getDataset(args[0]);
World world = context.getAgent().world();
Reference patchVar = ((org.nlogo.nvm.Argument)args[1]).getReference();
Envelope gisEnvelope = GISExtension.getState().getTransformation().getEnvelope(world);
Dimension gridSize = new Dimension(world.worldWidth(), world.worldHeight());
RasterDataset resampledDataset = dataset.resample(new GridDimensions(gridSize, gisEnvelope));
WritableRaster raster = resampledDataset.getRaster();
int px = world.minPxcor();

for(int ix = 0; px <= world.maxPxcor(); ++ix) {
    int py = world.minPycor();

    for(int iy = raster.getHeight() - 1; py <= world.maxPycor(); --iy) {
        Patch p = world.fastGetPatchAt(px, py);
        p.setVariable(patchVar.vn(), Double.valueOf(raster.getSampleDouble(ix, iy, 0)));
        ++py;
    }

    ++px;
}
```


```java
public RasterDataset resample(GridDimensions toDimensions) {
    if (toDimensions.equals(this._dimensions)) {
        return this;
    } else {
            double targetLeft = (toDimensions.getLeft() - this._dimensions.getLeft()) / this._dimensions.getCellWidth();
            double targetRight = (toDimensions.getRight() - this._dimensions.getLeft()) / this._dimensions.getCellWidth();
            double targetBottom = (toDimensions.getBottom() - this._dimensions.getBottom()) / this._dimensions.getCellHeight();
            double targetTop = (toDimensions.getTop() - this._dimensions.getBottom()) / this._dimensions.getCellHeight();
            ColorModel srcCM = new ValueColorModel(this._raster);
            RenderedImage srcImg = new BufferedImage(srcCM, this._raster, false, null);
            Insets border = new Insets(0, 0, 0, 0);
            BorderExtender borderExtender = new BorderExtenderConstant(new double[]{Double.NaN});
        if (targetLeft < 0.0) {
            border.left = -((int)StrictMath.floor(targetLeft));
        }

        if (targetRight > (double)this._dimensions.getGridWidth()) {
            border.right = (int)StrictMath.ceil(targetRight) - this._dimensions.getGridWidth();
        }

        if (targetBottom < 0.0) {
            border.bottom = -((int)StrictMath.floor(targetBottom));
        }

        if (targetTop > (double)this._dimensions.getGridHeight()) {
            border.top = (int)StrictMath.ceil(targetTop) - this._dimensions.getGridHeight();
        }

        if (border.left > 0 || border.right > 0 || border.bottom > 0 || border.top > 0) {
            ParameterBlock pb = new ParameterBlock();
            pb.addSource(srcImg);
            pb.add(Integer.valueOf(border.left));
            pb.add(Integer.valueOf(border.right));
            pb.add(Integer.valueOf(border.top));
            pb.add(Integer.valueOf(border.bottom));
            pb.add(borderExtender);
            srcImg = createRendering(JAI.create("border", pb), srcCM);
        }

        double flippedTopY = (double)(this._dimensions.getGridHeight() + border.top + border.bottom) - targetTop;
        if (targetLeft > 0.0
        || targetRight < (double)this._dimensions.getGridWidth()
        || targetBottom > 0.0
        || targetTop < (double)this._dimensions.getGridHeight()) {
            float left = Math.max((float)targetLeft, 0.0F);
            float bottom = Math.max((float)flippedTopY, 0.0F);
            float right = Math.min((float)(targetRight - targetLeft), (float)(this._dimensions.getGridWidth() - 1));
            float top = Math.min((float)(targetTop - targetBottom), (float)(this._dimensions.getGridHeight() - 1));
            ParameterBlock pb = new ParameterBlock();
            pb.addSource(srcImg);
            pb.add(Float.valueOf(left));
            pb.add(Float.valueOf(bottom));
            pb.add(Float.valueOf(right));
            pb.add(Float.valueOf(top));
            pb.add(borderExtender);
            srcImg = createRendering(JAI.create("crop", pb), srcCM);
        }

        double scaleX = (double)toDimensions.getGridWidth() / (double)srcImg.getWidth();
        double scaleY = (double)toDimensions.getGridHeight() / (double)srcImg.getHeight();
        double transX = (this._dimensions.getLeft() - toDimensions.getLeft()) / toDimensions.getCellWidth();
        double transY = (this._dimensions.getTop() - toDimensions.getBottom()) / toDimensions.getCellHeight();
        transY = (double)toDimensions.getGridHeight() - transY;
        ParameterBlock pb = new ParameterBlock();
        pb.addSource(srcImg);
        pb.add(Float.valueOf((float)scaleX));
        pb.add(Float.valueOf((float)scaleY));
        pb.add(Float.valueOf((float)transX));
        pb.add(Float.valueOf((float)transY));
        pb.add(this._interpolation);
        RenderingHints hints = new RenderingHints(JAI.KEY_BORDER_EXTENDER, borderExtender);
        RenderedOp dstImg = JAI.create("scale", pb, hints);
        WritableRaster wr = srcCM.createCompatibleWritableRaster(dstImg.getWidth(), dstImg.getHeight());
        dstImg.copyData(wr);
        return new RasterDataset(toDimensions, wr);
    }
}
```


```java
AsciiGrid Raster

width = ncols
height = nrows


new DataBufferDouble(width * height);
DataBuffer.offset = 0;
DataBuffer.banks = 1;
DataBuffer.dataType = 5; // some tag value for double (5)

new BandedSampleModel(dataType, width, height, numBands: 1);
new ComponentSampleModel(dataType, width, height, pixelStride: 1, scanlineStride: width, numBands: 1);
bankIndices = int[numBands]; // array index = value
bandOffsets = int[numBands]; // all zero
scanlineStride = width
pixelStride = 1
numBanks = 1
numBands = 1
width = w = // width in pixels
height = h = // height in pixels



public double getElemDouble(bank: 0, int i) {
    // should just be the entire image as a single array
    // loaded with: result.setElemDouble(index++, value); which means, row major storage, basically how its read in the file.
    return data[i]
}

public double getSampleDouble(int x, int y, int bank: 0, DataBuffer data) {
    // check x,y in range of width,height
    return data.getElemDouble(0, y * width + x)
}
```

# set-world-envelope
given the raw union of envelopes from the raster datasets
```java
World w = context.getAgent().world();
GISExtension.getState()
.setTransformation(
    new CoordinateTransformation(
        EnvelopeLogoListFormat.getInstance().parse(args[0].getList()),
        new Envelope((double)w.minPxcor(), (double)w.maxPxcor(), (double)w.minPycor(), (double)w.maxPycor()),
        true
    )
);
```