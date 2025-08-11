package flash.geom {
    import flash.display.DisplayObject;
    import flash.geom.Matrix3D;
    import flash.geom.Point;

    [Ruffle(InstanceAllocator)]
    public class PerspectiveProjection {
        public function PerspectiveProjection() {
            super();
        }

        public native function get fieldOfView():Number;

        public native function set fieldOfView(value:Number):void;

        public native function get focalLength():Number;

        public native function set focalLength(value:Number):void;

        public native function get projectionCenter():Point;

        public native function set projectionCenter(value:Point):*;

        public native function toMatrix3D():Matrix3D;
    }
}
