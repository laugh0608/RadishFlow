import javax.swing.*;
import java.awt.*;
import java.awt.event.ActionEvent;
import java.awt.event.ActionListener;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileWriter;
import java.io.PrintWriter;

public class HeaterGUI extends JFrame implements ActionListener
{
    // 创建两个文本显示示例
    private JTextField textField_T, textField_P;
    // 创建两个文本输入框实例
    private JComboBox comboBox_T, comboBox_P;
    // 创建两个按钮实例
    private JButton button_Submit, button_Cancel;

    public HeaterGUI(){
        // 窗体大小
        setSize(400,200);
        // 窗体标题
        setTitle("Laugh Heater");

        // 创建一个UI表格，3行1列
        setLayout(new GridLayout(3,1));
        // 创建输入温度面板
        JPanel panel_T = new JPanel();
        // 添加显示文本
        panel_T.add(new JLabel("请输入出口温度："));
        // 添加一个文本框
        textField_T = new JTextField();
        // 设置文本框的宽度
        textField_T.setColumns(10);
        // 将文本框添加到温度面板中
        panel_T.add(textField_T);
        // 添加一个下拉列表用来放温度单位
        String[] temperatureUnits = {"K", "C", "F"};
        comboBox_T = new JComboBox<>(temperatureUnits);
        // 将下拉列表添加到温度面板中
        panel_T.add(comboBox_T);
        // 将输入温度面板按次序添加在第一行第列
        add(panel_T);

        // 创建输入压力面板
        JPanel panel_P = new JPanel();
        // 添加显示文本
        panel_P.add(new JLabel("请输入出口压力："));
        // 添加一个文本框
        textField_P = new JTextField();
        // 设置文本框的宽度
        textField_P.setColumns(10);
        // 将文本框添加到压力面板中
        panel_P.add(textField_P);
        // 添加一个下拉列表用来放压力单位
        String[] pressureUnits = {"Pa", "bar", "atm"};
        comboBox_P = new JComboBox<>(pressureUnits);
        // 将下拉列表添加到压力面板中
        panel_P.add(comboBox_P);
        // 将输入温度面板按次序添加在第二行第一列
        add(panel_P);

        // 创建一个按钮面板
        JPanel panel_SubmitAndCancel = new JPanel();
        // 创建确认按钮
        button_Submit = new JButton("确定");
        // 给按钮绑定事件监听
        button_Submit.addActionListener(this);
        // 将确认按钮添加到按钮面板中
        panel_SubmitAndCancel.add(button_Submit);
        // 创建取消按钮
        button_Cancel = new JButton("取消");
        // 给按钮绑定事件监听
        button_Cancel.addActionListener(this);
        // 将取消按钮添加到按钮面板中
        panel_SubmitAndCancel.add(button_Cancel);
        // 将按钮面板按次序添加到表格中第三行第一列
        add(panel_SubmitAndCancel);

        // 关闭按钮
        setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        // 在创建窗口后，调用 setVisible(true) 来显示窗口
        setVisible(true);
    }

    public static void main(String[] args) {
        new HeaterGUI();
    }

    @Override
    public void actionPerformed(ActionEvent e) {
        // 提交按钮
        if(e.getSource() == button_Submit){
            // 获取输入框中的温度值，统一转换单位为K
            double temperature_Out = Double.parseDouble(textField_T.getText());
            // 单位转换C->K
            if(comboBox_T.getSelectedIndex() == 1) temperature_Out = temperature_Out + 273.15;
            // 单位转换F->K
            else if (comboBox_T.getSelectedIndex() == 2) temperature_Out = (temperature_Out-32)*5/9+273.15;

            // 获取输入框中的压力值，统一转换单位为Pa
            double pressure_Out = Double.parseDouble(textField_P.getText());
            // 单位转换bar->Pa
            if (comboBox_P.getSelectedIndex() == 1) pressure_Out = pressure_Out*100000;
            // 单位转换atm->Pa
            else if (comboBox_P.getSelectedIndex()== 2) pressure_Out = pressure_Out*101325;

            // 将输入结果输出到指定路径下的data.txt文件中暂存
            try {
                // 创建一个txt文件，注意这里的路径当前执行的用户要有权限进行访问
                PrintWriter pw = new PrintWriter(new File("C:/Users/laugh/Downloads/laughHeater_data.txt"));
                // 保存温度值
                pw.println(temperature_Out);
                // 保存压力值
                pw.println(pressure_Out);
                pw.close();
            } catch (FileNotFoundException ex) {
                throw new RuntimeException(ex);
            }
        }
        // 取消按钮
        else if (e.getSource() == button_Cancel){
            // 如果点击取消按钮，则直接返回空，并触发exit关闭窗口
        }
        System.exit(0);
    }
}
